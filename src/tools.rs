use std::collections::{HashMap, HashSet};
use std::panic;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::ThreadPool;
use regex::Regex;
use reqwest::Url;
use robotstxt::DefaultMatcher;
use rusqlite::Connection;
use scraper::{Html, Selector};

use crate::config;
use crate::data;
use crate::http;
use crate::sqlite;


pub(crate) fn debug_log(debug: bool, log_message: &str) {
    if debug {
        println!("{}", log_message);
    }
}

pub(crate) fn is_robots_txt_blocked(config: &config::Config, db_conn: &Arc<Mutex<Connection>>, url: Url, referrer_url: &String) -> bool {
    // We have to cache this or its death for performance.
    let domain = url.domain().unwrap();
    let robots_txt = INMEMORY_CACHE.get(domain).unwrap_or_else(|| {
        let curr_robots_txt = match reqwest::blocking::get(format!("https://{}/robots.txt", domain)) {
            Ok(response) => response.text().unwrap(),
            Err(_) => match reqwest::blocking::get(format!("http://{}/robots.txt", domain)) {
                Ok(response) => match response.text() {
                    Ok(text) => text,
                    Err(_) => return "".to_string(),
                },
                Err(_) => return "".to_string(),
            },
        };
        INMEMORY_CACHE.set(domain.to_string(), curr_robots_txt.clone());
        curr_robots_txt
    });

    // This can panic if the robots.txt is invalid
    let result = panic::catch_unwind(|| {
        !DefaultMatcher::default().allowed_by_robots(&robots_txt, config::USER_AGENTS.into_iter().collect(), url.as_str())
    });
    let blocked = match result {
        Ok(blocked) => blocked,
        Err(_) => {
            debug_log(config.debug, &format!("An error occurred while checking if the URL {} is allowed by robots.txt", url));
            false
        }
    };
    if blocked {
        let formatted_link_url = format_url_for_storage(url.to_string());
        let formatted_referrer_url = format_url_for_storage(referrer_url.to_string());
        if let Err(e) = sqlite::mark_url_blocked(&mut db_conn.lock().unwrap(), &formatted_link_url, &formatted_referrer_url) {
            debug_log(config.debug, &format!("Failed to mark URL {} as blocked in SQLite: {}", url, e));
        }
    }
    blocked
}

// Save image data, and the links to the database
pub(crate) fn save_image_links(config: &config::Config, pool: &Arc<ThreadPool>, site_urls: &data::SiteUrls, db_conn: &Arc<Mutex<Connection>>, target_url: &Url) {
    pool.install(|| {
        site_urls.img_urls.clone().into_par_iter().for_each(|url| {
            let result = http::fetch_image(config, &url);
            let image_data = match &result {
                Ok(content) => content,
                Err(e) => {
                    debug_log(config.debug, &format!("Failed to fetch image from {}: {}", url, e));
                    return
                }
            };
            // Check if we got any image data
            if image_data.len() == 0 {
                return
            }

            // Expect an image name at the end of the url, grab it
            let name = url.path_segments().and_then(|segments| segments.last()).unwrap_or(".jpg");
            let _ = sqlite::insert_image(&mut db_conn.lock().unwrap(), &format_url_for_storage(target_url.to_string()),&format_url_for_storage(url.to_string()), image_data, &name.to_string(), result.is_ok())
                .map_err(|e| debug_log(config.debug, &format!("Failed to insert image into SQLite: {}", e)));
        });
    });
}

// Parse HTML content into a scraper::Html object
pub(crate) fn parse_html(html: &str) -> Result<Html, Box<dyn std::error::Error>> {
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
pub(crate) fn extract_links(doc: &Html) -> Result<data::SiteLinks, Box<dyn std::error::Error>> {
    let link_links = extract_attributes(doc, "a[href]", "href");
    let img_links = extract_attributes(doc, "img[src]", "src");

    Ok(data::SiteLinks {
        link_links,
        img_links,
    })
}

// Save some recursion, remove duplicates and links we've seen.
pub(crate) fn filter_links(config: &config::Config, links: Vec<String>, seen: &Arc<Mutex<HashSet<String>>>, db_conn: &Arc<Mutex<Connection>>, referrer_url: &String) -> HashSet<Url> {
    let mut links_set: HashSet<Url> = HashSet::new();
    links_set.extend(links.into_iter().filter_map(|mut link: String| {
        // Handle any links that are relative paths
        link = match http::handle_relative_paths(config, &link, referrer_url) {
            Ok(value) => value,
            Err(_) => {
                return None;
            },
        };
        
        // Check if the link is valid
        let (link_url, is_valid) = is_valid_site(config, &link);
        if is_valid {
            if let Some(link_url) = link_url {
                let formatted_link_url = format_url_for_storage(link_url.to_string());
                if seen.lock().unwrap().contains(&formatted_link_url) {
                    // Check if we have already seen this URL
                    debug_log(config.debug, &format!("Ignoring previously seen URL: {}", formatted_link_url));
                    return None;
                } else if sqlite::is_previously_completed_url(&mut db_conn.lock().unwrap(), &formatted_link_url).unwrap().unwrap() {
                    // Check if this URL has already been completed
                    seen.lock().unwrap().insert(formatted_link_url.clone());
                    debug_log(config.debug, &format!("Ignoring completed URL: {}", formatted_link_url));
                    return None;
                } else if config.respect_robots && is_robots_txt_blocked(config, db_conn, link_url.clone(), &referrer_url) {
                    // Check if this URL should be ignored due to robots.txt
                    seen.lock().unwrap().insert(formatted_link_url.clone());
                    debug_log(config.debug, &format!("Ignoring robots.txt blocked URL: {}", link_url));
                    return None;
                }
                seen.lock().unwrap().insert(formatted_link_url.clone());
                return Some(link_url);
            }
        }
        None
    }));
    links_set
}

pub(crate) fn filter_links_to_urls(config: &config::Config, links: data::SiteLinks, seen: &Arc<Mutex<HashSet<String>>>, db_conn: &Arc<Mutex<Connection>>, referrer_url: &String) -> data::SiteUrls {
    let link_links_set = filter_links(config, links.link_links, seen, db_conn, referrer_url);
    let img_links_set = filter_links(config, links.img_links, seen, db_conn, referrer_url);
    // Convert the HashSet to a Vec, preventing duplicates.
    let link_urls_vec: Vec<Url> = link_links_set.into_iter().collect();
    let img_urls_vec: Vec<Url> = img_links_set.into_iter().collect();
    data::SiteUrls {
        link_urls: link_urls_vec,
        img_urls: img_urls_vec,
    }
}

// Determine if a site or relative path is valid.
pub(crate) fn is_valid_site(config: &config::Config, url: &str) -> (Option<Url>, bool) {
    if let Ok(parsed_url) = Url::parse(&url) {
        // Check if the domain of the URL is in the list of permitted domains.
        if let Some(domain) = parsed_url.domain() {
            if (config.free_crawl || config.permitted_domains.iter().any(|d| domain.eq(d)))
                  && !config.blacklist_domains.iter().any(|d| domain.eq(d)) {
                return (Some(parsed_url), true);
            } else {
                // If the domain isn't in the list of permitted domains..
                debug_log(config.debug, &format!("Domain {} isn't in the list of permitted domains: {:?}", domain, parsed_url));
                return (Some(parsed_url), false);
            }
        } else {
            // If the URL doesn't have a domain..
            debug_log(config.debug, &format!("URL {} doesn't have a domain", url));
            return (Some(parsed_url), false);
        }
    }
    (None, false)
}

// Run this before storing a url. Prevent storing the same url multiple times.
pub(crate) fn format_url_for_storage(url: String) -> String{
    // capture everything after the protocol and before the query
    let re = Regex::new(r"^https?://(www\.)?([^?]*).*").unwrap();
    let formatted_url = re.replace(&url, "$2").trim_end_matches('/').to_string();
    formatted_url
}

// A simple in-memory cache that uses a HashMap and a Mutex.
// This is used to cache robots.txt files for each run. 
struct InMemoryCache {
    map: Mutex<HashMap<String, String>>,
}
impl InMemoryCache {
    fn new() -> InMemoryCache {
        InMemoryCache {
            map: Mutex::new(HashMap::new()),
        }
    }
    fn get(&self, key: &str) -> Option<String> {
        let map = self.map.lock().unwrap();
        map.get(key).cloned()
    }
    fn set(&self, key: String, value: String) {
        let mut map = self.map.lock().unwrap();
        map.insert(key, value);
    }
}
lazy_static! {
    static ref INMEMORY_CACHE: InMemoryCache = InMemoryCache::new();
}



// Apparently pulling this in is a common practice. I didnt write it.
// Using this similar to defer in Go, we can ensure that a function is run when the current scope is exited.
pub(crate) struct Defer<F: FnOnce()> {
    f: Option<F>,
}
impl<F: FnOnce()> Defer<F> {
    pub(crate) fn new(f: F) -> Defer<F> {
        Defer { f: Some(f) }
    }
}
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(site_urls.link_links, vec!["https://www.cnn.com"]);
    }

    #[test]
    fn test_is_valid_site() {
        let url = "https://www.cnn.com";
        let config: config::Config = config::Config::new();
        let (_, is_valid) = is_valid_site(&config, url);
        assert!(is_valid);
    }
}