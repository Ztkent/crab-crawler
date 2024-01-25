
// Site Settings
pub(crate) const STARTING_URL: &str = "https://www.cnn.com";
pub(crate) const PERMITTED_DOMAINS: [&str; 1] = ["www.cnn.com"];
pub(crate) const BLACKLIST_DOMAINS: [&str; 0] = [];
pub(crate) const FREE_CRAWL: bool = false;

// Crawler Settings
pub(crate) const MAX_URLS_TO_VISIT: usize = 250;
pub(crate) const MAX_THREADS: usize = 5;

// Logging Options
pub(crate) const DEBUG: bool = false;
pub(crate) const LIVE_LOGGING: bool = true;

// Database Settings
pub(crate) const SQLITE_ENABLED: bool = true;
pub(crate) const SQLITE_PATH: &str = "db/crawl_results.db";
