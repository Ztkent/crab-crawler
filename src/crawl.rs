use reqwest::blocking::Client;
use reqwest::Error;
use reqwest::Url;
use rusqlite::Connection;
use scraper::{Html, Selector};
use std::sync::{Arc, Mutex};
use rayon::ThreadPool;
use rayon::prelude::*;
use regex::Regex;
use chrono::{Local, DateTime};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::constants as consts;
use crate::sqlite;

#[derive(Clone)]
pub(crate) struct VisitedSite {
    url: String,
    referrer: String,
    visited_at: DateTime<Local>,
}
impl VisitedSite {
    // This is a constructor method that creates a new instance of Visited.
    pub fn new(url: String, referrer: String, visited_at: DateTime<Local>) -> Self {
        Self { url, referrer, visited_at }
    }

    // These are getter methods that return the value of each field.
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

// Crawl a website, collecting links.
fn crawl_website(db_conn: Arc<Mutex<Connection>>, pool: Arc<ThreadPool>, target_url: String, referer_url: String) -> bool {
    if URLS_VISITED.load(Ordering::SeqCst) >= consts::MAX_URLS_TO_VISIT {
        // Base Case
        return false;
    }
    
    // Remove the protocol, trailing slash, and tracking information from the URL
    let re = Regex::new(r"^https?://(www\.)?([^?]*).*").unwrap();
    let visited_url = re.replace(&target_url.clone(), "$2").trim_end_matches('/').to_string();
    
    { // Limit the scope of the mutex lock
        let mut conn: std::sync::MutexGuard<'_, Connection> = db_conn.lock().unwrap();
        if referer_url != "STARTING_URL" && sqlite::is_previously_visited_url(&mut conn, &visited_url).unwrap().unwrap() {
            // Check if we've already visited this URL, then skip it.
            return true;
        } else {
            // Otherwise, add the URL to the visited set
            if consts::LIVE_LOGGING {
                println!("Visiting {}", visited_url);
            }

            let visited_site = VisitedSite::new(visited_url.clone(), referer_url.clone(), Local::now());
            // Add the visited URL to the database
            URLS_VISITED.fetch_add(1, Ordering::SeqCst);
            if consts::SQLITE_ENABLED {
                if let Err(e) = sqlite::insert_visited_site(&mut conn, visited_site.clone()) {
                    if consts::DEBUG {
                        eprintln!("Failed to insert visited URL {} into SQLite: {}",visited_url.clone(), e);
                    }
                }
            }
        }
    }
    // Fetch the HTML content of the page
    let html = match fetch_html(&target_url) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Failed to fetch HTML from {}: {}", target_url, e);
            return true;
        }
    };

    // Parse the HTML content into a Html object
    let doc = match parse_html(&html) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("Failed to parse HTML: {}", e);
            return true;
        }
    };

    // Extract the links from the Html object
    let links = match extract_links(&doc) {
        Ok(links) => links,
        Err(e) => {
            eprintln!("Failed to extract links from {}: {}", target_url, e);
            return true;
        }
    };

    // Recursively crawl each link
    // This is thread-safe, and will never run more than MAX_THREADS concurrent requests.
    pool.install(|| {
        // Handle the links
        let complete = links.link_urls.into_par_iter().try_for_each(|link| {
            if is_valid_site(&link) {
                let pool = Arc::clone(&pool);
                if !crawl_website(db_conn.clone(), pool, link, visited_url.clone()) {
                    return Err(());
                }
            }
            return Ok(());
        });

        // Mark the page as finished in sqlite
        if let Ok(_) = complete {
            match sqlite::mark_url_complete(&mut db_conn.lock().unwrap(), &visited_url){
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to mark URL {} as complete in SQLite: {}", visited_url, e);
                }
            };
        }
    });
    return true;
}
    


// Fetch HTML from a given URL
fn fetch_html(url: &str) -> Result<String, Error> {
    // Create a new HTTP client
    let client = Client::new();
    
    // Send a GET request to the specified URL and get a response
    let res = client.get(url).send().map_err(|err| {
        eprintln!("Failed to send request to {}: {}", url, err);
        return err;
    })?;
    
    // Get the body of the response as a String
    let body = res.text().map_err(|err| {
        eprintln!("Failed to read response from {}: {}", url, err);
        return err;
    })?;
    
    // Return the body of the response
   return Ok(body);
}

// Parse HTML content into a scraper::Html object
fn parse_html(html: &str) -> Result<Html, Box<dyn std::error::Error>> {
    // Parse the HTML content
    let document = Html::parse_document(html);
    
    // Return the parsed HTML
    return Ok(document);
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
    return urls
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

fn is_valid_site(url: &str) -> bool {
    if let Ok(parsed_url) = Url::parse(url) {
        // Check if the domain of the URL is in the list of permitted domains.
        if let Some(domain) = parsed_url.domain() {
            if (consts::FREE_CRAWL || consts::PERMITTED_DOMAINS.iter().any(|&d| domain.eq(d)))
                  && !consts::BLACKLIST_DOMAINS.iter().any(|&d| domain.eq(d)) {
                return true;
            } else {
                // If the domain isn't in the list of permitted domains, print an error message, and all of the other parsed_url fields
                if consts::DEBUG {
                    eprintln!("Domain {} isn't in the list of permitted domains: {:?}", domain, parsed_url);
                }
                return false;
            }
        } else {
            // If the URL doesn't have a domain, print an error message, and all of the other parsed_url fields
            if consts::DEBUG {
                eprintln!("URL {} doesn't have a domain: {:?}", url, parsed_url);
            }
            return false;
        }
    }
    return false;
}

pub(crate) fn timed_crawl_website(db_conn: Connection, pool: Arc<ThreadPool>, url: String) {
    let start = Local::now();
    let db_conn = Arc::new(Mutex::new(db_conn));
    crawl_website(db_conn, pool, url, "STARTING_URL".to_string());
    let duration = Local::now().signed_duration_since(start);
    println!("Time elapsed in crawl_website() is: {:?}", duration);
}
