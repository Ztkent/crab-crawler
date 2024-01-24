# 🦀 crab-crawler 🦀 

This is a rust web crawler, designed to collect training data.  
It that starts from a given URL and follows all links to whitelisted domains.

### Constants

- `PERMITTED_DOMAINS`: An array of domain names that the crawler is allowed to visit.
- `BLACKLIST_DOMAINS`: An array of domain names that the crawler is banned from visiting.
- `FREE_CRAWL`: A boolean that allows the crawler to visit any domain not in the blacklist.
- `STARTING_URL`: The URL that the crawler starts from.
- `MAX_URLS_TO_VISIT`: The maximum number of URLs that the crawler will visit before stopping.
- `MAX_THREADS`: The maximum number of threads that the crawler will use.
- `DEBUG`: A boolean that enables debug output.
- `LIVE_LOGGING`: A boolean that will log all URLs as they are visited.

### Output

The program outputs the URLs of all visited pages to the console. If an error occurs, it outputs an error message.

### Implementation

The crawler uses a thread pool to visit multiple URLs concurrently. It keeps track of visited URLs in a thread-safe hash set. It uses the `reqwest` crate to send HTTP requests, and the `scraper` crate to parse HTML and extract links.
