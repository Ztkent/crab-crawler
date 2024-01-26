use reqwest::{Error, Url, header::{self, HeaderValue}};
use rusqlite::Connection;
use scraper::{Html, Selector};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use rayon::{ThreadPool, prelude::*};
use regex::Regex;
use chrono::{Local, DateTime};
use rand::seq::SliceRandom;

use crate::constants as consts;
use crate::sqlite;
use crate::tools;

#[derive(Clone)]
pub(crate) struct VisitedSite {
    url: String,
    referrer: String,
    visited_at: DateTime<Local>,
}
impl VisitedSite {
    pub fn new(url: String, referrer: String, visited_at: DateTime<Local>) -> Self {
        Self { url, referrer, visited_at }
    }
    pub fn url(&self) -> &String {
        &self.url
    }
    pub fn referrer(&self) -> &String {
        &self.referrer
    }
    pub fn visited_at(&self) -> &DateTime<Local> {
        &self.visited_at
    }
}
pub(crate) static URLS_VISITED: AtomicUsize = AtomicUsize::new(0);

// Struct to hold all the different types of URLs
struct SiteUrls {
    link_urls: Vec<String>,
    // img_urls: Vec<String>,
    // stylesheet_urls: Vec<String>,
    // script_urls: Vec<String>,
    // object_data_urls: Vec<String>,
    // embed_urls: Vec<String>,
    // video_urls: Vec<String>,
    // audio_urls: Vec<String>,
    // source_urls: Vec<String>,
}

// Recursively crawl a website, with Depth-First Search.
fn crawl_website_dfs(db_conn: Arc<Mutex<Connection>>, pool: Arc<ThreadPool>, target_url: Url, referer_url: String) -> bool {
    if URLS_VISITED.load(Ordering::SeqCst) >= consts::MAX_URLS_TO_VISIT {
        // Base Case
        return false;
    }
    
    let re = Regex::new(r"^https?://(www\.)?([^?]*).*").unwrap();
    let visited_url = re.replace(target_url.as_str(), "$2").trim_end_matches('/').to_string();
    
    { // Scope the mutable borrow of db_conn
        let mut conn = db_conn.lock().unwrap();
        if referer_url != "STARTING_URL" && sqlite::is_previously_visited_url(&mut conn, &visited_url).unwrap().unwrap() {
            tools::debug_log(&format!("Ignoring previously visited URL: {}", visited_url));
            return true;
        }
        
        // Insert the visited URL into SQLite
        let visited_site = VisitedSite::new(visited_url.clone(), referer_url.clone(), Local::now());
        URLS_VISITED.fetch_add(1, Ordering::SeqCst);
        if let Err(e) = sqlite::insert_visited_site(&mut conn, visited_site.clone()) {
            tools::debug_log(&format!("Failed to insert visited URL {} into SQLite: {}", visited_url, e));
        }
    }

    // Fetch the HTML content of the page
    let html = match fetch_html(target_url.clone()) {
        Ok(html) => html,
        Err(e) => {
            tools::debug_log(&format!("Failed to fetch HTML from {}: {}", target_url, e));
            return true;
        }
    };

    // Parse the HTML content into a Html object
    let doc = match parse_html(&html) {
        Ok(doc) => doc,
        Err(e) => {
            tools::debug_log(&format!("Failed to parse HTML from {}: {}", target_url, e));
            return true;
        }
    };

    // Extract the links from the Html object
    let links = match extract_links(&doc) {
        Ok(links) => links,
        Err(e) => {
            tools::debug_log(&format!("Failed to extract links from {}: {}", target_url, e));
            return true;
        }
    };

    // Recursively crawl each link
    // This is thread-safe, and will never run more than MAX_THREADS concurrent requests.
    let success = Arc::new(Mutex::new(true));
    pool.install(|| {
        // Handle the links
        let complete = links.link_urls.into_par_iter().try_for_each(|link| {
            let (link_url, is_valid) = is_valid_site(&link);
            if is_valid {
                if let Some(link_url) = link_url {
                    let pool = Arc::clone(&pool);
                    if !crawl_website_dfs(db_conn.clone(), pool, link_url, visited_url.clone()) {
                        *success.lock().unwrap() = false;
                        return Err(());
                    }
                }
            }
            Ok(())
        });

        // Mark the page as finished in sqlite
        if complete.is_ok() {
            if let Err(e) = sqlite::mark_url_complete(&mut db_conn.lock().unwrap(), &visited_url) {
                tools::debug_log(&format!("Failed to mark URL {} as complete in SQLite: {}", visited_url, e));
            }
        }
    });

    // Check if we successfully set all of the child pages.
    // If so, then we can mark this page as complete
    if success.lock().unwrap().clone() {
        return true;
    } 
    // If we were not able to complete crawling this entire page and its child pages,
    // then we need to let the recursive parent know by returning false.
    false
}
    


// Fetch HTML from a given URL
fn fetch_html(url: Url) -> Result<String, Error> {
    // Create a new HTTP client
    let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(consts::CRAWLER_REQUEST_TIMEOUT))
    .build()
    .unwrap();
    
    // Handle the robots.txt file, skipping any URLs that are disallowed
    if consts::RESPECT_ROBOTS {
        // We will want to do this with a cached version of the robots.txt file. 
        // let robots_url: String = format!("https://{}/robots.txt", url);
    }

    // Randomly pick a user agent from the list
    let mut user_agent = consts::USER_AGENT_CHROME;
    if consts::ROTATE_USER_AGENT {
        user_agent = consts::USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
    }
    
    // Print the URL that we are visiting
    if consts::LIVE_LOGGING {
        println!("Visiting {}", url);
    }

    // Send a GET request to the specified URL and get a response
    let res = client.get(url.clone())
        .header(header::USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
        .send()
        .map_err(|err| {
            err
        })?;
    
    // Get the body of the response as a String
    let body = res.text().map_err(|err| {
        err
    })?;
    
    // Return the body of the response
   Ok(body)
}

// Parse HTML content into a scraper::Html object
fn parse_html(html: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    Ok(document)
}

// Helper function to extract attributes from elements that match a selector
fn extract_attributes(doc: &Html, selector_str: &str, attr: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let selector = Selector::parse(selector_str).unwrap();
    for element in doc.select(&selector) {
        if let Some(url) = element.value().attr(attr) {
            urls.push(url.to_string());
        }
    }
    urls
}

// Extract all links and image URLs from parsed HTML
fn extract_links(doc: &Html) -> Result<SiteUrls, Box<dyn std::error::Error>> {
    let link_urls = extract_attributes(doc, "a[href]", "href");
    // let img_urls = extract_attributes(doc, "img[src]", "src");
    // let stylesheet_urls = extract_attributes(doc, "link[rel=stylesheet][href]", "href");
    // let script_urls = extract_attributes(doc, "script[src]", "src");
    // let object_data_urls = extract_attributes(doc, "object[data]", "data");
    // let embed_urls = extract_attributes(doc, "embed[src]", "src");
    // let video_urls = extract_attributes(doc, "video[src]", "src");
    // let audio_urls = extract_attributes(doc, "audio[src]", "src");
    // let source_urls = extract_attributes(doc, "source[src]", "src");

    Ok(SiteUrls {
        link_urls,
        // img_urls,
        // stylesheet_urls,
        // script_urls,
        // object_data_urls,
        // embed_urls,
        // video_urls,
        // audio_urls,
        // source_urls,
    })
}

fn is_valid_site(url: &str) -> (Option<Url>, bool) {
    if let Ok(parsed_url) = Url::parse(url) {
        // Check if the domain of the URL is in the list of permitted domains.
        if let Some(domain) = parsed_url.domain() {
            if (consts::FREE_CRAWL || consts::PERMITTED_DOMAINS.iter().any(|&d| domain.eq(d)))
                  && !consts::BLACKLIST_DOMAINS.iter().any(|&d| domain.eq(d)) {
                return (Some(parsed_url), true);
            } else {
                // If the domain isn't in the list of permitted domains, print an error message, and all of the other parsed_url fields
                tools::debug_log(&format!("Domain {} isn't in the list of permitted domains: {:?}", domain, parsed_url));
                return (Some(parsed_url), false);
            }
        } else {
            // If the URL doesn't have a domain, print an error message, and all of the other parsed_url fields
            tools::debug_log(&format!("URL {} doesn't have a domain", url));
            return (Some(parsed_url), false);
        }
    }
    (None, false)
}

pub(crate) fn timed_crawl_website(db_conn: Connection, pool: Arc<ThreadPool>, url: Url) {
    let start = Local::now();
    let db_conn = Arc::new(Mutex::new(db_conn));
    crawl_website_dfs(db_conn, pool, url, "STARTING_URL".to_string());
    let duration: chrono::Duration = Local::now().signed_duration_since(start);
    println!("Time elapsed in crawl_website() is: {:?}", duration);
}
