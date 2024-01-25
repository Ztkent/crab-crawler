use reqwest::blocking::Client;
use reqwest::Error;
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rayon::ThreadPool;
use rayon::prelude::*;
use std::time::Instant;
use regex::Regex;
use crate::constants as consts;

#[derive(Clone)]
pub(crate) struct VisitedSite {
    url: String,
    referrer: String,
    visited_at: Instant,
}
impl VisitedSite {
    // This is a constructor method that creates a new instance of Visited.
    pub fn new(url: String, referrer: String, visited_at: Instant) -> Self {
        Self { url, referrer, visited_at }
    }

    // These are getter methods that return the value of each field.
    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn referrer(&self) -> &String {
        &self.referrer
    }

    pub fn visited_at(&self) -> &Instant {
        &self.visited_at
    }
}

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

// Crawl a website, collecting links.
fn crawl_website(pool:Arc<ThreadPool>, target_url: String, referer_url: String, visited: Arc<Mutex<HashMap<String, VisitedSite>>>) {
    if visited.lock().unwrap().len() >= consts::MAX_URLS_TO_VISIT {
        // Base Case
        return;
    }
    
    // Remove the protocol, trailing slash, and tracking information from the URL
    let re = Regex::new(r"^https?://(www\.)?([^?]*).*").unwrap();
    let visited_url = re.replace(&target_url.clone(), "$2").trim_end_matches('/').to_string();
    if visited_url != consts::STARTING_URL && visited.lock().unwrap().contains_key(&visited_url) {
        // If the URL is in the visited set, skip it.
        return;
    } else {
        // Otherwise, add the URL to the visited set
        if consts::LIVE_LOGGING {
            println!("Visiting {}", visited_url);
        }
        let visited_site = VisitedSite::new(visited_url.clone(), referer_url.clone(), Instant::now());
        visited.lock().unwrap().insert(visited_url, visited_site);
    }

    // Fetch the HTML content of the page
    let html = match fetch_html(&target_url) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Failed to fetch HTML from {}: {}", target_url, e);
            return;
        }
    };

    // Parse the HTML content into a Html object
    let doc = match parse_html(&html) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("Failed to parse HTML: {}", e);
            return;
        }
    };

    // Extract the links from the Html object
    let links = match extract_links(&doc) {
        Ok(links) => links,
        Err(e) => {
            eprintln!("Failed to extract links from {}: {}", target_url, e);
            return;
        }
    };

    // Recursively crawl each link
    // This is thread-safe, and will never run more than MAX_THREADS concurrent requests.
    pool.install(|| {
        // Handle the links
        links.link_urls.into_par_iter().for_each(|link| {
            if is_valid_site(&link) {
                let visited = Arc::clone(&visited);
                let pool = Arc::clone(&pool);
                crawl_website(pool, link, target_url.clone(),visited);
            }
        });
    });
}

pub(crate) fn timed_crawl_website(pool: Arc<ThreadPool>, url: String, visited: Arc<Mutex<HashMap<String, VisitedSite>>>) {
    let start = Instant::now();
    crawl_website(pool, url, "STARTING_URL".to_string(), visited);
    let duration = start.elapsed();
    println!("Time elapsed in crawl_website() is: {:?}", duration);
}
