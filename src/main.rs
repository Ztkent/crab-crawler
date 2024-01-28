use std::sync::Arc;
use rayon::{ThreadPool, ThreadPoolBuilder};
use reqwest::Url;
use std::{
    thread,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

mod crawl;
mod tools;
mod sqlite;
mod constants;
use constants as consts;

/*
This is a rust web crawler. It starts from a given URL and follows all links to whitelisted domains.
With some adjustments, it can be used to collect training data.

Constants:
// Site Settings
- `PERMITTED_DOMAINS`: An array of domain names that the crawler is allowed to visit.
- `BLACKLIST_DOMAINS`: An array of domain names that the crawler is banned from visiting.
- `STARTING_URL`: The URL that the crawler starts from.

// Crawler Settings
- `MAX_URLS_TO_VISIT`: The maximum number of URLs that the crawler will visit before stopping.
- `MAX_THREADS`: The maximum number of threads that the crawler will use.
- `CRAWLER_TIMEOUT`: The maximum time the crawler will run.
- `CRAWLER_REQUEST_TIMEOUT`: The maximum time the crawler will wait for a request to return.
- `CRAWLER_REQUEST_DELAY_MS`: The time each crawler thread will wait between visiting sites.

// Logging Options
- `DEBUG`: A boolean that enables debug output.
- `LIVE_LOGGING`: A boolean that will log all URLs as they are visited.

// Database Settings
- `SQLITE_ENABLED`: A boolean that enables pushing results to SQLite. 
- `SQLITE_PATH`: The path to the SQLite database file.

// Features
- `FREE_CRAWL`: A boolean that, if true, allows the crawler to visit any domain. This will respect the Blacklist.
- `ROTATE_USER_AGENT`: A boolean that enables user agent rotation.
- `RESPECT_ROBOTS`: A boolean that enables respecting robots.txt files.

Output:
- The program outputs the URLs of all visited pages to a sqlite db.
- If `DEBUG` is true, the program outputs debug information to the console.
- If `LIVE_LOGGING` is true, the program outputs the URLs of all visited pages to the console.
*/

fn main() {
    // Connect to the SQLite database and run any migrations
    let conn = match sqlite::connect_sqlite_and_migrate() {
        Ok(connection) => connection.unwrap(),
        Err(e) => {
            eprintln!("Failed to connect to SQLite and migrate: {}", e);
            return;
        }
    };

    // Start crawling, with a timeout.
    let starting_url = Url::parse(consts::STARTING_URL).expect("Failed to parse starting URL");
    let pool: Arc<ThreadPool> = Arc::new(ThreadPoolBuilder::new().num_threads(consts::MAX_THREADS).build().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        crawl::timed_crawl_website(conn, pool, starting_url);
        tx.send(()).ok();
    });
    
    loop {
        // Wait for the job to complete, or a timeout.
        match rx.recv_timeout(Duration::from_secs(consts::CRAWLER_TIMEOUT)) {
            Ok(()) => {
                println!("Crawler thread finished successfully.");
                break;
            }
            Err(RecvTimeoutError::Timeout) => {
                eprintln!("Crawler thread timed-out after {:?} seconds. Aborting...", consts::CRAWLER_TIMEOUT);
                break;
            }
            Err(RecvTimeoutError::Disconnected) => {
                eprintln!("Crawler thread disconnected. Aborting...");
                break;
            }
        }
    }

    // Print the number of URLs visited
    println!("Visited {} URLs.", crawl::URLS_VISITED.load(std::sync::atomic::Ordering::SeqCst));
    if consts::SQLITE_ENABLED {
        println!("DB Contains {:?} URLs, {:?} complete.", sqlite::connect_and_get_total_rows().unwrap(), sqlite::connect_and_get_completed_rows().unwrap());
    }
}