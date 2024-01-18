# 🦀 crab-crawler 🦀 

This is a rust web crawler. It that starts from a given URL and follows all links to whitelisted domains.

### Constants

- `PERMITTED_DOMAINS`: An array of domain names that the crawler is allowed to visit. The crawler will only follow links that lead to these domains.
- `BLACKLIST_DOMAINS`: An array of domain names that the crawler is banned from visiting.
- `FREE_CRAWL`: A boolean that allows the crawler to visit any domain not in the blacklist.
- `STARTING_URL`: The URL that the crawler starts from.
- `MAX_URLS_TO_VISIT`: The maximum number of URLs that the crawler will visit before stopping.
- `MAX_THREADS`: The maximum number of threads that the crawler will use.
- `DEBUG`: A boolean that, if true, enables debug output.

### Output

The program outputs the URLs of all visited pages to the console. If an error occurs, it outputs an error message.

### Implementation

The crawler uses a thread pool to visit multiple URLs concurrently. It keeps track of visited URLs in a thread-safe hash set. It uses the `reqwest` crate to send HTTP requests, and the `scraper` crate to parse HTML and extract links.
