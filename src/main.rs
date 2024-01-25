use std::sync::Arc;
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
- The program outputs the URLs of all visited pages to a sqlite db. If an error occurs, it outputs an error message.
- The program outputs the number of URLs visited to the console.
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

    // Start crawling
    let pool: Arc<ThreadPool> = Arc::new(ThreadPoolBuilder::new().num_threads(consts::MAX_THREADS).build().unwrap());
    crawl::timed_crawl_website(conn, pool, consts::STARTING_URL.to_string());

    // Print the number of URLs visited
    println!("Visited {} URLs", crawl::URLS_VISITED.load(std::sync::atomic::Ordering::SeqCst));
}