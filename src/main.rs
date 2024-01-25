use reqwest::blocking::Client;
use reqwest::Error;
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rayon::ThreadPoolBuilder;
use rayon::ThreadPool;
use rayon::prelude::*;
use std::time::Instant;
use regex::Regex;
use rusqlite::{Connection, Result};

const STARTING_URL: &str = "https://www.cnn.com";
const PERMITTED_DOMAINS: [&str; 1] = ["www.cnn.com"];
const BLACKLIST_DOMAINS: [&str; 0] = [];
const FREE_CRAWL: bool = true;

const MAX_URLS_TO_VISIT: usize = 25;
const MAX_THREADS: usize = 5;
const DEBUG: bool = false;
const LIVE_LOGGING: bool = false;

/*
This is a rust web crawler. It starts from a given URL and follows all links to whitelisted domains.
With some adjustments, it can be used to collect training data.

Constants:
- `PERMITTED_DOMAINS`: An array of domain names that the crawler is allowed to visit. The crawler will only follow links that lead to these domains.
- `BLACKLIST_DOMAINS`: An array of domain names that the crawler is banned from visiting.
- `FREE_CRAWL`: A boolean that, if true, allows the crawler to visit any domain. This will respect the Blacklist.
- `STARTING_URL`: The URL that the crawler starts from.
- `MAX_URLS_TO_VISIT`: The maximum number of URLs that the crawler will visit before stopping.
- `MAX_THREADS`: The maximum number of threads that the crawler will use.
- `DEBUG`: A boolean that enables debug output.
- `LIVE_LOGGING`: A boolean that will log all URLs as they are visited.

Output:
- The program outputs the URLs of all visited pages to the console. If an error occurs, it outputs an error message.

The crawler uses a thread pool to visit multiple URLs concurrently.
It keeps track of visited URLs in a thread-safe hash map. 
It uses the `reqwest` crate to send HTTP requests, and `scraper` crate to parse HTML and extract links.
*/

#[derive(Clone)]
struct Visited {
    url: String,
    referrer: String,
    visited_at: Instant,
}

// Struct to hold all the different types of URLs
struct Urls {
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

fn main() {
    // Connect to sqlite
    let resultsDB = match Connection::open("src/crawl_results.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("{:?}", e);
            return
        }
    };

    // Start crawling
    let visited: Arc<Mutex<HashMap<String, Visited>>> = Arc::new(Mutex::new(HashMap::new()));    let pool: Arc<ThreadPool> = Arc::new(ThreadPoolBuilder::new().num_threads(MAX_THREADS).build().unwrap());
    timed_crawl_website(pool,STARTING_URL.to_string(), visited.clone());

    // Convert the HashMap to a Vec so that we can sort it
    let mut visits: Vec<(String, Visited)> = visited.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    visits.sort_by(|a, b| a.1.visited_at.cmp(&b.1.visited_at));
    
    // Print the visited URLs
    println!("Visited URLs:");
    for visit in visits {
        println!("{} - > {}", visit.1.referrer, visit.1.url);
    }
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
fn extract_links(doc: &Html) -> Result<Urls, Box<dyn std::error::Error>> {
    let link_urls = extract_attributes(doc, "a[href]", "href");
    // let img_urls = extract_attributes(doc, "img[src]", "src");
    // let stylesheet_urls = extract_attributes(doc, "link[rel=stylesheet][href]", "href");
    // let script_urls = extract_attributes(doc, "script[src]", "src");
    // let object_data_urls = extract_attributes(doc, "object[data]", "data");
    // let embed_urls = extract_attributes(doc, "embed[src]", "src");
    // let video_urls = extract_attributes(doc, "video[src]", "src");
    // let audio_urls = extract_attributes(doc, "audio[src]", "src");
    // let source_urls = extract_attributes(doc, "source[src]", "src");

    Ok(Urls {
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
            if (FREE_CRAWL || PERMITTED_DOMAINS.iter().any(|&d| domain.eq(d)))
                  && !BLACKLIST_DOMAINS.iter().any(|&d| domain.eq(d)) {
                return true;
            } else {
                // If the domain isn't in the list of permitted domains, print an error message, and all of the other parsed_url fields
                if DEBUG {
                    eprintln!("Domain {} isn't in the list of permitted domains: {:?}", domain, parsed_url);
                }
                return false;
            }
        } else {
            // If the URL doesn't have a domain, print an error message, and all of the other parsed_url fields
            if DEBUG {
                eprintln!("URL {} doesn't have a domain: {:?}", url, parsed_url);
            }
            return false;
        }
    }
    return false;
}

// Crawl a website, collecting links.
fn crawl_website(pool:Arc<ThreadPool>, target_url: String, referer_url: String, visited: Arc<Mutex<HashMap<String, Visited>>>) {
    if visited.lock().unwrap().len() >= MAX_URLS_TO_VISIT {
        // Base Case
        return;
    }
    
    // Remove the protocol, trailing slash, and tracking information from the URL
    let re = Regex::new(r"^https?://(www\.)?([^?]*).*").unwrap();
    let visited_url = re.replace(&target_url.clone(), "$2").trim_end_matches('/').to_string();
    if visited_url != STARTING_URL && visited.lock().unwrap().contains_key(&visited_url) {
        // If the URL is in the visited set, skip it.
        return;
    } else {
        // Otherwise, add the URL to the visited set
        if LIVE_LOGGING {
            println!("Visiting {}", visited_url);
        }
        let visited_site = Visited {
            url: target_url.clone(),
            referrer: referer_url.clone(),
            visited_at: Instant::now(),
        };
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

fn timed_crawl_website(pool: Arc<ThreadPool>, url: String, visited: Arc<Mutex<HashMap<String, Visited>>>) {
    let start = Instant::now();
    crawl_website(pool, url, "STARTING_URL".to_string(), visited);
    let duration = start.elapsed();
    println!("Time elapsed in crawl_website() is: {:?}", duration);
}
