use std::sync::atomic::AtomicUsize;

use chrono::{DateTime, Local};
use reqwest::Url;

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
pub(crate) struct SiteUrls {
    pub(crate) link_urls: Vec<Url>,
    pub(crate) img_urls: Vec<Url>,
    // video_urls: Vec<Url>,
    // audio_urls: Vec<Url>,
    // source_urls: Vec<Url>,
}

pub(crate) struct SiteLinks {
    pub(crate) link_links: Vec<String>,
    pub(crate) img_links: Vec<String>,
}