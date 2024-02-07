use serde::Deserialize;
use serde_json::Value;
use std::fs;
use crate::constants;
use crate::tools;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Site Settings
    pub starting_url: String,
    pub permitted_domains: Vec<String>,
    pub blacklist_domains: Vec<String>,
    pub free_crawl: bool,

    // Crawler Settings
    pub max_urls_to_visit: usize,
    pub max_threads: usize,
    pub rotate_user_agents: bool,
    pub respect_robots: bool,
    pub crawler_timeout: u64,
    pub crawler_request_timeout: u64,
    pub crawler_request_delay_ms: u64,

    // Data Collection Options
    pub collect_html: bool,
    pub collect_images: bool,

    // Logging Options
    pub debug: bool,
    pub live_logging: bool,

    // Database Settings
    pub sqlite_enabled: bool,
    pub sqlite_path: String,

    // Features
    pub user_agents: Vec<String>,
    pub log_relative_paths: bool,
}

impl Config {
    pub fn new(path: String) -> Self {
        let mut config = Config { // Default Crawler Config
            starting_url: constants::STARTING_URL.to_string(),
            permitted_domains: constants::PERMITTED_DOMAINS.iter().map(|s| s.to_string()).collect(),
            blacklist_domains: constants::BLACKLIST_DOMAINS.iter().map(|s| s.to_string()).collect(),
            free_crawl: constants::FREE_CRAWL,
            max_urls_to_visit: constants::MAX_URLS_TO_VISIT,
            max_threads: constants::MAX_THREADS,
            rotate_user_agents: constants::ROTATE_USER_AGENTS,
            respect_robots: constants::RESPECT_ROBOTS,
            crawler_timeout: constants::CRAWLER_TIMEOUT,
            crawler_request_timeout: constants::CRAWLER_REQUEST_TIMEOUT,
            crawler_request_delay_ms: constants::CRAWLER_REQUEST_DELAY_MS,
            collect_html: constants::COLLECT_HTML,
            collect_images: constants::COLLECT_IMAGES,
            debug: constants::DEBUG,
            live_logging: constants::LIVE_LOGGING,
            sqlite_enabled: constants::SQLITE_ENABLED,
            sqlite_path: constants::SQLITE_PATH.to_string(),
            user_agents: constants::USER_AGENTS.iter().map(|&s| s.to_string()).collect(),
            log_relative_paths: constants::LOG_RELATIVE_PATHS,
        };

        // Attempt to read and parse the configuration file
        if path.is_empty() {
            tools::debug_log(true, "No config file provided, using defaults.");
            return config;
        }
        
        match fs::read_to_string(path.clone()) {
            Ok(contents) => match serde_json::from_str::<Value>(&contents) {
                Ok(json_config) => {
                    // Overwrite the defaults with the values from the file
                    tools::debug_log(true, &format!("Using provided config file: {}", path));
                    if let Some(starting_url) = json_config.get("starting_url").and_then(Value::as_str) {
                        config.starting_url = starting_url.to_string();
                    }
                    if let Some(permitted_domains) = json_config.get("permitted_domains").and_then(Value::as_array) {
                        config.permitted_domains = permitted_domains.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect();
                    }
                    if let Some(blacklist_domains) = json_config.get("blacklist_domains").and_then(Value::as_array) {
                        config.blacklist_domains = blacklist_domains.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect();
                    }
                    if let Some(free_crawl) = json_config.get("free_crawl").and_then(Value::as_bool) {
                        config.free_crawl = free_crawl;
                    }
                    if let Some(max_urls_to_visit) = json_config.get("max_urls_to_visit").and_then(Value::as_u64) {
                        config.max_urls_to_visit = max_urls_to_visit as usize;
                    }
                    if let Some(max_threads) = json_config.get("max_threads").and_then(Value::as_u64) {
                        config.max_threads = max_threads as usize;
                    }
                    if let Some(rotate_user_agents) = json_config.get("rotate_user_agents").and_then(Value::as_bool) {
                        config.rotate_user_agents = rotate_user_agents;
                    }
                    if let Some(respect_robots) = json_config.get("respect_robots").and_then(Value::as_bool) {
                        config.respect_robots = respect_robots;
                    }
                    if let Some(crawler_timeout) = json_config.get("crawler_timeout").and_then(Value::as_u64) {
                        config.crawler_timeout = crawler_timeout;
                    }
                    if let Some(crawler_request_timeout) = json_config.get("crawler_request_timeout").and_then(Value::as_u64) {
                        config.crawler_request_timeout = crawler_request_timeout;
                    }
                    if let Some(crawler_request_delay_ms) = json_config.get("crawler_request_delay_ms").and_then(Value::as_u64) {
                        config.crawler_request_delay_ms = crawler_request_delay_ms;
                    }
                    if let Some(collect_html) = json_config.get("collect_html").and_then(Value::as_bool) {
                        config.collect_html = collect_html;
                    }
                    if let Some(collect_images) = json_config.get("collect_images").and_then(Value::as_bool) {
                        config.collect_images = collect_images;
                    }
                    if let Some(debug) = json_config.get("debug").and_then(Value::as_bool) {
                        config.debug = debug;
                    }
                    if let Some(live_logging) = json_config.get("live_logging").and_then(Value::as_bool) {
                        config.live_logging = live_logging;
                    }
                    if let Some(sqlite_enabled) = json_config.get("sqlite_enabled").and_then(Value::as_bool) {
                        config.sqlite_enabled = sqlite_enabled;
                    }
                    if let Some(sqlite_path) = json_config.get("sqlite_path").and_then(Value::as_str) {
                        config.sqlite_path = sqlite_path.to_string();
                    }
                    if let Some(user_agents) = json_config.get("user_agents").and_then(Value::as_array) {
                        config.user_agents = user_agents.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect();
                    }
                    if let Some(log_relative_paths) = json_config.get("log_relative_paths").and_then(Value::as_bool) {
                        config.log_relative_paths = log_relative_paths;
                    }
                },
                Err(err) => tools::debug_log(true, &format!("Failed to parse config file, using defaults: {}", err)),
            },
            Err(err) => tools::debug_log(true, &format!("Failed to read config file, using defaults: {}", err)),
        }
        
        config
    }
}
