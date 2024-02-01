// Site Settings
pub(crate) const STARTING_URL: &str = "https://www.cnn.com";
pub(crate) const PERMITTED_DOMAINS: [&str; 1] = ["www.cnn.com"];
pub(crate) const BLACKLIST_DOMAINS: [&str; 0] = [];
pub(crate) const FREE_CRAWL: bool = true;
//google.com/landing, https://workspace.google.com/, https://www.cnn.com/sitemap.html

// Crawler Settings
pub(crate) const MAX_URLS_TO_VISIT: usize = 1000;
pub(crate) const MAX_THREADS: usize = 4;
pub(crate) const ROTATE_USER_AGENTS: bool = true;
pub(crate) const RESPECT_ROBOTS: bool = true;
pub(crate) const CRAWLER_TIMEOUT: u64 = 1200; 
pub(crate) const CRAWLER_REQUEST_TIMEOUT: u64 = 5; 
pub(crate) const CRAWLER_REQUEST_DELAY_MS: u64 = 5000; 

// Data Collection Options
pub(crate) const COLLECT_HTML: bool = false;
pub(crate) const COLLECT_IMAGES: bool = false;
// pub(crate) const COLLECT_PDFS: bool = true;
// pub(crate) const COLLECT_SCREENSHOTS: bool = true;

// Logging Options
pub(crate) const DEBUG: bool = true;
pub(crate) const LIVE_LOGGING: bool = true;

// Database Settings
pub(crate) const SQLITE_ENABLED: bool = true;
pub(crate) const SQLITE_PATH: &str = "db/crawl_results.db";

// User Agents
pub(crate) const USER_AGENT_CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3";
const USER_AGENT_FIREFOX: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0";
const USER_AGENT_SAFARI: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/603.3.8 (KHTML, like Gecko) Version/10.1.2 Safari/603.3.8";
const USER_AGENT_IE: &str = "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; AS; rv:11.0) like Gecko";
const USER_AGENT_EDGE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/64.0.3282.140 Safari/537.36 Edge/17.17134";
const USER_AGENT_OPERA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36 OPR/64.0.3417.54";
const USER_AGENT_BRAVE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.108 Safari/537.36 Brave/78.1.3.15";
pub(crate) const USER_AGENTS: [&str; 7] = [USER_AGENT_CHROME, USER_AGENT_FIREFOX, USER_AGENT_SAFARI, USER_AGENT_IE, USER_AGENT_EDGE, USER_AGENT_OPERA, USER_AGENT_BRAVE];


// Testing
pub(crate) const LOG_RELATIVE_PATHS: bool = true;