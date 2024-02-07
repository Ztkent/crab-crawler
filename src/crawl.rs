use reqwest::Url;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, atomic::Ordering};
use rayon::{ThreadPool, prelude::*};
use chrono::Local;
use std::collections::HashSet;

use crate::config;
use crate::sqlite;
use crate::tools;
use crate::http;
use crate::data;

pub struct Crawler {
    config: config::Config,
    db_conn: Arc<Mutex<Connection>>,
    pool: Arc<ThreadPool>,
    seen: Arc<Mutex<HashSet<String>>>,
}

impl Crawler {
    pub fn new(config:  config::Config, db_conn: Connection, pool: Arc<ThreadPool>) -> Self {
        let db_conn = Arc::new(Mutex::new(db_conn));
        let seen = Arc::new(Mutex::new(HashSet::new()));
        Crawler { config, db_conn, pool, seen }
    }

    // Recursively crawl a website, with Depth-First Search.
    pub fn crawl_website_dfs(&self, target_url: &Url, referrer_url: &String) -> bool {
        if data::URLS_VISITED.load(Ordering::SeqCst) >= self.config.max_urls_to_visit {
            // Base Case
            return false;
        }
        
        // Format the visited URL for storage and comparison
        let formatted_target_url = tools::format_url_for_storage(target_url.to_string());
        let formatted_referrer_url = tools::format_url_for_storage(referrer_url.clone());
        { // Scope the mutable borrow of db_conn, otherwise it will stay in scope due to recursion below.
            let mut conn = self.db_conn.lock().unwrap();
            // Store the visited URL
            let visited_site = data::VisitedSite::new(formatted_target_url.clone(), formatted_referrer_url.clone(), Local::now());
            data::URLS_VISITED.fetch_add(1, Ordering::SeqCst);
            if let Err(e) = sqlite::insert_visited_site(&mut conn, visited_site.clone()) {
                tools::debug_log(self.config.debug, &format!("Failed to insert visited URL {} into SQLite: {}", formatted_target_url, e));
            }
        }
    
        // Set the delay before continuing after the request is complete.
        let _defer = tools::Defer::new(|| {
            std::thread::sleep(std::time::Duration::from_millis(self.config.crawler_request_delay_ms));
        });
    
        // Fetch the HTML content of the page
        if self.config.live_logging {
            println!("Visiting {} from {}", target_url, referrer_url);
        }
        let html = match http::fetch_html(&self.config, &self.db_conn, target_url.clone()) {
            Ok(html) => html,
            Err(e) => {
                tools::debug_log(self.config.debug, &format!("Failed to fetch HTML from {}: {}", target_url, e));
                return true;
            }
        };
    
        // Parse the HTML content into a Html object
        let doc = match tools::parse_html(&html) {
            Ok(doc) => doc,
            Err(e) => {
                tools::debug_log(self.config.debug, &format!("Failed to parse HTML from {}: {}", target_url, e));
                return true;
            }
        };
    
        // Extract the links from the Html object
        // TODO: Handle Sitemaps
        let site_links = match tools::extract_links(&doc){
            Ok(links) => links,
            Err(e) => {
                tools::debug_log(self.config.debug, &format!("Failed to extract links from {}: {}", target_url, e));
                return true;
            }
        };
    
        // Filter links to only include those that are valid, and not already seen or completed.
        let site_urls = tools::filter_links_to_urls(&self.config, site_links, &self.seen, &self.db_conn, &target_url.to_string());
        // Fetch any images from the page
        if self.config.collect_images {
            tools::save_image_links(&self.config, &self.pool, &site_urls, &self.db_conn, target_url);
        }
        
        // Recursively crawl each link
        // This is thread-safe, and will never run more than MAX_THREADS concurrent requests.
        let success = Arc::new(Mutex::new(true));
        self.pool.install(|| {
            // Handle the links
            let complete = site_urls.link_urls.into_par_iter().try_for_each(|url| {
                        if !self.crawl_website_dfs( &url, &formatted_target_url) {
                            *success.lock().unwrap() = false;
                            return Err(());
                        }
                Ok(())
            });
    
            // Mark the page as finished in sqlite
            if complete.is_ok() {
                if let Err(e) = sqlite::mark_url_complete(&mut self.db_conn.lock().unwrap(), &formatted_target_url) {
                    tools::debug_log(self.config.debug, &format!("Failed to mark URL {} as complete in SQLite: {}", formatted_target_url, e));
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

    pub fn timed_crawl_website(&self, url: Url) {
        let start = Local::now();
        self.crawl_website_dfs(&url, &"STARTING_URL".to_string());
        let duration: chrono::Duration = Local::now().signed_duration_since(start);
        println!("Time elapsed in crawl_website() is: {:?}", duration);
    }
}