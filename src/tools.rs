use std::collections::HashMap;
use std::sync::Mutex;
use std::panic;
use regex::Regex;
use reqwest::Url;
use robotstxt::DefaultMatcher;
use crate::constants as consts;
use crate::sqlite;
use lazy_static::lazy_static;
use std::sync::Arc;
use rusqlite::Connection;

pub(crate) fn debug_log(log_message: &str) {
    if consts::DEBUG {
        println!("{}", log_message);
    }
}

pub(crate) fn is_robots_txt_blocked(db_conn: &Arc<Mutex<Connection>>, url: Url, referrer_url: &String) -> bool {
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
        !DefaultMatcher::default().allowed_by_robots(&robots_txt, consts::USER_AGENTS.into_iter().collect(), url.as_str())
    });
    let blocked = match result {
        Ok(blocked) => blocked,
        Err(_) => {
            debug_log(&format!("An error occurred while checking if the URL {} is allowed by robots.txt", url));
            false
        }
    };
    if blocked {
        if let Err(e) = sqlite::mark_url_blocked(&mut db_conn.lock().unwrap(), &url.to_string(), referrer_url) {
            debug_log(&format!("Failed to mark URL {} as blocked in SQLite: {}", url, e));
        }
    }
    blocked
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