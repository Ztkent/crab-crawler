# 🦀 crab-crawler 🦀 
This is a rust web crawler, it is designed to collect training data.  

## Configuration

### Site Settings
- **STARTING_URL**: The URL that the crawler starts from.
- **PERMITTED_DOMAINS**: An array of domain names that the crawler is allowed to visit.
- **BLACKLIST_DOMAINS**: An array of domain names that the crawler is banned from visiting.

### Crawler Settings
- **MAX_URLS_TO_VISIT**: The maximum number of URLs that the crawler will visit before stopping.
- **MAX_THREADS**: The maximum number of threads that the crawler will use.
- **CRAWLER_REQUEST_TIMEOUT**: The maximum time the crawler will wait for a request to return.

### Logging Options
- **DEBUG**: A boolean that enables debug output.
- **LIVE_LOGGING**: A boolean that will log all URLs as they are visited.

### Database Settings
- **SQLITE_ENABLED**: A boolean that enables pushing results to SQLite.
- **SQLITE_PATH**: The path to the SQLite database file.

### Features
- **FREE_CRAWL**: A boolean that, if true, allows the crawler to visit any domain. This will respect the Blacklist.
- **ROTATE_USER_AGENT**: A boolean that enables user agent rotation.
- **RESPECT_ROBOTS**: A boolean that enables respecting robots.txt files.

## Output
The crawler outputs the URLs of all visited pages to the console.

### SQLite
The crawler can also output the URLs of all visited pages to a SQLite database.  
To enable this:
- set `SQLITE_ENABLED` to `true`
- set `SQLITE_PATH` to the path of the SQLite database file

## Implementation
- starts from a given URL and follows all links to whitelisted domains.  
- uses a thread pool to visit multiple URLs concurrently.
- swaps the user agent between requests.
- stores selected data in a sqlite database for review.

