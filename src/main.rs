use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rayon::ThreadPoolBuilder;
use rayon::ThreadPool;

mod crawl;
mod sqlite;
mod constants;
use constants as consts;

/*
This is a rust web crawler. It starts from a given URL and follows all links to whitelisted domains.
With some adjustments, it can be used to collect training data.

Constants:
- `PERMITTED_DOMAINS`: An array of domain names that the crawler is allowed to visit.
- `BLACKLIST_DOMAINS`: An array of domain names that the crawler is banned from visiting.
- `FREE_CRAWL`: A boolean that, if true, allows the crawler to visit any domain. This will respect the Blacklist.
- `STARTING_URL`: The URL that the crawler starts from.
- `MAX_URLS_TO_VISIT`: The maximum number of URLs that the crawler will visit before stopping.
- `MAX_THREADS`: The maximum number of threads that the crawler will use.
- `DEBUG`: A boolean that enables debug output.
- `LIVE_LOGGING`: A boolean that will log all URLs as they are visited.
- `SQLITE_ENABLED`: A boolean that enables pushing results to SQLite.
- `SQLITE_PATH`: The path to the SQLite database file.

Output:
- The program outputs the URLs of all visited pages to the console. If an error occurs, it outputs an error message.
*/

fn main() {
    // Connect to the SQLite database and run any migrations
    let _ = match sqlite::connect_sqlite_and_migrate() {
        Ok(connection) => connection,
        Err(e) => {
            eprintln!("Failed to connect to SQLite and migrate: {}", e);
            return;
        }
    };

    // Start crawling
    let visited: Arc<Mutex<HashMap<String, crawl::VisitedSite>>> = Arc::new(Mutex::new(HashMap::new()));    let pool: Arc<ThreadPool> = Arc::new(ThreadPoolBuilder::new().num_threads(consts::MAX_THREADS).build().unwrap());
    crawl::timed_crawl_website(pool,consts::STARTING_URL.to_string(), visited.clone());

    // Sort the visited URLs 
    let mut visits: Vec<(String,crawl::VisitedSite)> = visited.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    visits.sort_by(|a, b| a.1.visited_at().cmp(b.1.visited_at()));
    
    // Print the visited URLs
    println!("Visited URLs:");
    for visit in visits {
        println!("{} - > {}", visit.1.referrer(), visit.1.url());
    }
}