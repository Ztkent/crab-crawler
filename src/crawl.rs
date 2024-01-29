use reqwest::{Error, Url, header::{self, HeaderValue}};
use rusqlite::Connection;
use scraper::{Html, Selector};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use rayon::{ThreadPool, prelude::*};
use chrono::{Local, DateTime};
use rand::seq::SliceRandom;
use std::collections::HashSet;

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
    link_urls: Vec<Url>,
    // img_urls: Vec<Url>,
    // video_urls: Vec<Url>,
    // audio_urls: Vec<Url>,
    // source_urls: Vec<Url>,
}

struct SiteLinks {
    link_urls: Vec<String>,
}

// Recursively crawl a website, with Depth-First Search.
fn crawl_website_dfs(db_conn: Arc<Mutex<Connection>>, pool: Arc<ThreadPool>, seen: Arc<Mutex<HashSet<String>>>, target_url: &Url, referrer_url: &String) -> bool {
    if URLS_VISITED.load(Ordering::SeqCst) >= consts::MAX_URLS_TO_VISIT {
        // Base Case
        return false;
    }
    
    // Format the visited URL for storage and comparison
    let formatted_target_url = tools::format_url_for_storage(target_url.to_string());
    { // Scope the mutable borrow of db_conn, otherwise it will stay in scope due to recursion below.
        let mut conn = db_conn.lock().unwrap();
        // Store the visited URL
        let visited_site = VisitedSite::new(formatted_target_url.clone(), referrer_url.clone(), Local::now());
        URLS_VISITED.fetch_add(1, Ordering::SeqCst);
        seen.lock().unwrap().insert(formatted_target_url.clone());
        if let Err(e) = sqlite::insert_visited_site(&mut conn, visited_site.clone()) {
            tools::debug_log(&format!("Failed to insert visited URL {} into SQLite: {}", formatted_target_url, e));
        }
    }

    // Set the delay before continuing after the request is complete.
    let _defer = tools::Defer::new(|| {
        std::thread::sleep(std::time::Duration::from_millis(consts::CRAWLER_REQUEST_DELAY_MS));
    });

    // Fetch the HTML content of the page
    if consts::LIVE_LOGGING {
        println!("Visiting {} from {}", target_url, referrer_url);
    }
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
    let site_links = match extract_links(&doc){
        Ok(links) => links,
        Err(e) => {
            tools::debug_log(&format!("Failed to extract links from {}: {}", target_url, e));
            return true;
        }
    };

    // Filter links to only include those that are valid, and not already seen or completed.
    let site_urls = filter_links_to_urls(site_links, &seen, &db_conn, referrer_url.clone());

    // Recursively crawl each link
    // This is thread-safe, and will never run more than MAX_THREADS concurrent requests.
    let success = Arc::new(Mutex::new(true));
    pool.install(|| {
        // Handle the links
        let complete = site_urls.link_urls.into_par_iter().try_for_each(|url| {
                    if !crawl_website_dfs(db_conn.clone(), pool.clone(), seen.clone(), &url, &formatted_target_url) {
                        *success.lock().unwrap() = false;
                        return Err(());
                    }
            Ok(())
        });

        // Mark the page as finished in sqlite
        if complete.is_ok() {
            if let Err(e) = sqlite::mark_url_complete(&mut db_conn.lock().unwrap(), &formatted_target_url) {
                tools::debug_log(&format!("Failed to mark URL {} as complete in SQLite: {}", formatted_target_url, e));
            }
        }
    });

    // Check if we successfully crawled all of the child pages.
    // If so, then we can mark this page as complete.
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

    // Randomly pick a user agent from the list
    let mut user_agent = consts::USER_AGENT_CHROME;
    if consts::ROTATE_USER_AGENTS {
        user_agent = consts::USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
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
fn extract_links(doc: &Html) -> Result<SiteLinks, Box<dyn std::error::Error>> {
    let link_urls = extract_attributes(doc, "a[href]", "href");
    // let img_urls = extract_attributes(doc, "img[src]", "src");
    // let stylesheet_urls = extract_attributes(doc, "link[rel=stylesheet][href]", "href");
    // let script_urls = extract_attributes(doc, "script[src]", "src");
    // let object_data_urls = extract_attributes(doc, "object[data]", "data");
    // let embed_urls = extract_attributes(doc, "embed[src]", "src");
    // let video_urls = extract_attributes(doc, "video[src]", "src");
    // let audio_urls = extract_attributes(doc, "audio[src]", "src");
    // let source_urls = extract_attributes(doc, "source[src]", "src");

    // TODO: Handle relative URLs.
    Ok(SiteLinks {
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

// Save some recursion, remove duplicates and links we've seen.
fn filter_links_to_urls(links: SiteLinks, seen: &Arc<Mutex<HashSet<String>>>, db_conn: &Arc<Mutex<Connection>>, referrer_url: String) -> SiteUrls {
    let mut link_urls_set = HashSet::new();
    link_urls_set.extend(links.link_urls.into_iter().filter_map(|link: String| {
        let (link_url, is_valid) = is_valid_site(&link);
        if is_valid {
            if let Some(link_url) = link_url {
                // Check if this URL should be ignored due to robots.txt
                if consts::RESPECT_ROBOTS && tools::is_robots_txt_blocked(link_url.clone()) {
                    if consts::DEBUG {
                        tools::debug_log(&format!("Ignoring robots.txt blocked URL: {}", link_url));
                    }
                    return None;
                }

                // Recrawl the starting URL, even if it is marked as complete.
                if referrer_url == "STARTING_URL" {
                    return Some(link_url);
                }
                
                // Check if we have already seen this URL, or if it is already marked as complete.
                let formatted_link_url = tools::format_url_for_storage(link_url.to_string());
                if seen.lock().unwrap().contains(&formatted_link_url) {
                    tools::debug_log(&format!("Ignoring previously seen URL: {}", formatted_link_url));
                    return None;
                } else if sqlite::is_previously_completed_url(&mut db_conn.lock().unwrap(), &formatted_link_url).unwrap().unwrap() {
                    seen.lock().unwrap().insert(formatted_link_url.clone());
                    tools::debug_log(&format!("Ignoring completed URL: {}", formatted_link_url));
                    return None;
                } 
                return Some(link_url);
            }
        }
        None
    }));
    // Convert the HashSet to a Vec, preventing duplicates.
    let link_urls_vec: Vec<_> = link_urls_set.into_iter().collect();
    SiteUrls {
        link_urls: link_urls_vec,
    }
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
    let seen: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    crawl_website_dfs(db_conn, pool, seen, &url, &"STARTING_URL".to_string());
    let duration: chrono::Duration = Local::now().signed_duration_since(start);
    println!("Time elapsed in crawl_website() is: {:?}", duration);
}


// tests/crawl_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_fetch_html() {
        let url = Url::parse("https://www.cnn.com").unwrap();
        let result = fetch_html(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_html() {
        let html = "<html><body><h1>Hello, world!</h1></body></html>";
        let result = parse_html(html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_attributes() {
        let html = "<html><body><a href='https://www.cnn.com'>Link</a></body></html>";
        let document = Html::parse_document(html);
        let result = extract_attributes(&document, "a[href]", "href");
        assert_eq!(result, vec!["https://www.cnn.com"]);
    }

    #[test]
    fn test_extract_links() {
        let html = "<html><body><a href='https://www.cnn.com'>Link</a></body></html>";
        let document = Html::parse_document(html);
        let result = extract_links(&document);
        assert!(result.is_ok());
        let site_urls = result.unwrap();
        assert_eq!(site_urls.link_urls, vec!["https://www.cnn.com"]);
    }

    #[test]
    fn test_is_valid_site() {
        let url = "https://www.cnn.com";
        let (_, is_valid) = is_valid_site(url);
        assert!(is_valid);
    }
}