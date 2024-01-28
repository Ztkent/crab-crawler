use std::collections::HashMap;
use std::sync::Mutex;
use reqwest::Url;
use robotstxt::DefaultMatcher;
use crate::constants as consts;
use lazy_static::lazy_static;

pub(crate) fn debug_log(log_message: &str) {
    if consts::DEBUG {
        eprintln!("{}", log_message);
    }
}

pub(crate) fn is_robots_txt_blocked(url: Url) -> bool {
    // We have to cache this or its death for performance.
    let domain = url.domain().unwrap();
    let robots_url: String = format!("https://{}/robots.txt", domain);
    let robots_txt = INMEMORY_CACHE.get(domain).unwrap_or_else(|| {
        let curr_robots_txt = reqwest::blocking::get(robots_url.as_str()).unwrap().text().unwrap();
        INMEMORY_CACHE.set(domain.to_string(), curr_robots_txt.clone());
        curr_robots_txt
    });

    let mut matcher = DefaultMatcher::default();
    !matcher.allowed_by_robots(&robots_txt, consts::USER_AGENTS.into_iter().collect(), url.as_str())
}

// Defer is a helper struct that allows us to run a function when the struct is dropped.
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