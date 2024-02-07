#[derive(Clone)]
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

    // User Agents
    pub user_agents: Vec<String>,

    // Testing
    pub log_relative_paths: bool,
}


impl Config {
    pub fn new() -> Self {
        Config { // Default Crawler Config
            starting_url: "https://www.cnn.com".to_string(),
            permitted_domains: vec!["www.cnn.com".to_string()],
            blacklist_domains: vec![],
            free_crawl: true,
            max_urls_to_visit: 1000,
            max_threads: 8,
            rotate_user_agents: true,
            respect_robots: true,
            crawler_timeout: 1200,
            crawler_request_timeout: 5,
            crawler_request_delay_ms: 5000,
            collect_html: false,
            collect_images: true,
            debug: false,
            live_logging: true,
            sqlite_enabled: true,
            sqlite_path: "db/crawl_results.db".to_string(),
            user_agents: USER_AGENTS.iter().map(|&s| s.to_string()).collect(),
            log_relative_paths: false,
        }
    }
    
    pub fn set_starting_url(&mut self, url: String) -> &mut Self {
        self.starting_url = url;
        self
    }

    pub fn set_permitted_domains(&mut self, domains: Vec<String>) -> &mut Self {
        self.permitted_domains = domains;
        self
    }

    pub fn set_blacklist_domains(&mut self, domains: Vec<String>) -> &mut Self {
        self.blacklist_domains = domains;
        self
    }

    pub fn set_free_crawl(&mut self, free_crawl: bool) -> &mut Self {
        self.free_crawl = free_crawl;
        self
    }

    pub fn set_max_urls_to_visit(&mut self, max: usize) -> &mut Self {
        self.max_urls_to_visit = max;
        self
    }

    pub fn set_max_threads(&mut self, max: usize) -> &mut Self {
        self.max_threads = max;
        self
    }

    pub fn set_rotate_user_agents(&mut self, rotate: bool) -> &mut Self {
        self.rotate_user_agents = rotate;
        self
    }

    pub fn set_respect_robots(&mut self, respect: bool) -> &mut Self {
        self.respect_robots = respect;
        self
    }

    pub fn set_crawler_timeout(&mut self, timeout: u64) -> &mut Self {
        self.crawler_timeout = timeout;
        self
    }

    pub fn set_crawler_request_timeout(&mut self, timeout: u64) -> &mut Self {
        self.crawler_request_timeout = timeout;
        self
    }

    pub fn set_crawler_request_delay_ms(&mut self, delay: u64) -> &mut Self {
        self.crawler_request_delay_ms = delay;
        self
    }

    pub fn set_collect_html(&mut self, collect: bool) -> &mut Self {
        self.collect_html = collect;
        self
    }

    pub fn set_collect_images(&mut self, collect: bool) -> &mut Self {
        self.collect_images = collect;
        self
    }

    pub fn set_debug(&mut self, debug: bool) -> &mut Self {
        self.debug = debug;
        self
    }

    pub fn set_live_logging(&mut self, live: bool) -> &mut Self {
        self.live_logging = live;
        self
    }

    pub fn set_sqlite_enabled(&mut self, enabled: bool) -> &mut Self {
        self.sqlite_enabled = enabled;
        self
    }

    pub fn set_sqlite_path(&mut self, path: String) -> &mut Self {
        self.sqlite_path = path;
        self
    }

    pub fn set_user_agents(&mut self, agents: Vec<String>) -> &mut Self {
        self.user_agents = agents;
        self
    }

    pub fn set_log_relative_paths(&mut self, log: bool) -> &mut Self {
        self.log_relative_paths = log;
        self
    }
}

pub const USER_AGENTS: [&str; 7] = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/603.3.8 (KHTML, like Gecko) Version/10.1.2 Safari/603.3.8",
    "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; AS; rv:11.0) like Gecko",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/64.0.3282.140 Safari/537.36 Edge/17.17134",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36 OPR/64.0.3417.54",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.108 Safari/537.36 Brave/78.1.3.15",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_setters() {
        let mut config: Config = Config::new();

        assert_eq!(config.starting_url, "https://www.cnn.com");
        assert_eq!(config.permitted_domains, vec!["www.cnn.com"]);
        assert_eq!(config.blacklist_domains, Vec::<String>::new());
        assert_eq!(config.free_crawl, true);
        assert_eq!(config.max_urls_to_visit, 1000);
        assert_eq!(config.max_threads, 8);
        assert_eq!(config.rotate_user_agents, true);
        assert_eq!(config.respect_robots, true);
        assert_eq!(config.crawler_timeout, 1200);
        assert_eq!(config.crawler_request_timeout, 5);
        assert_eq!(config.crawler_request_delay_ms, 5000);
        assert_eq!(config.collect_html, false);
        assert_eq!(config.collect_images, true);
        assert_eq!(config.debug, false);
        assert_eq!(config.live_logging, true);
        assert_eq!(config.sqlite_enabled, true);
        assert_eq!(config.sqlite_path, "db/crawl_results.db");
        assert_eq!(config.user_agents, USER_AGENTS.iter().map(|&s| s.to_string()).collect::<Vec<String>>());
        assert_eq!(config.log_relative_paths, false);

        config.set_starting_url("https://www.example.com".to_string());
        config.set_permitted_domains(vec!["www.example.com".to_string()]);
        config.set_blacklist_domains(vec!["www.blacklisted.com".to_string()]);
        config.set_free_crawl(false);
        config.set_max_urls_to_visit(500);
        config.set_max_threads(4);
        config.set_rotate_user_agents(false);
        config.set_respect_robots(false);
        config.set_crawler_timeout(600);
        config.set_crawler_request_timeout(10);
        config.set_crawler_request_delay_ms(10000);
        config.set_collect_html(true);
        config.set_collect_images(false);
        config.set_debug(true);
        config.set_live_logging(false);
        config.set_sqlite_enabled(false);
        config.set_sqlite_path("db/test.db".to_string());
        config.set_user_agents(vec!["TestAgent".to_string()]);
        config.set_log_relative_paths(true);

        assert_eq!(config.starting_url, "https://www.example.com");
        assert_eq!(config.permitted_domains, vec!["www.example.com"]);
        assert_eq!(config.blacklist_domains, vec!["www.blacklisted.com"]);
        assert_eq!(config.free_crawl, false);
        assert_eq!(config.max_urls_to_visit, 500);
        assert_eq!(config.max_threads, 4);
        assert_eq!(config.rotate_user_agents, false);
        assert_eq!(config.respect_robots, false);
        assert_eq!(config.crawler_timeout, 600);
        assert_eq!(config.crawler_request_timeout, 10);
        assert_eq!(config.crawler_request_delay_ms, 10000);
        assert_eq!(config.collect_html, true);
        assert_eq!(config.collect_images, false);
        assert_eq!(config.debug, true);
        assert_eq!(config.live_logging, false);
        assert_eq!(config.sqlite_enabled, false);
        assert_eq!(config.sqlite_path, "db/test.db");
        assert_eq!(config.user_agents, vec!["TestAgent"]);
        assert_eq!(config.log_relative_paths, true);
    }
}