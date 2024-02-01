# 🦀 data-crawler 🦀 
This is a rust web crawler, it is designed to collect training data from the web.  

## Configuration

### Site Settings
- **STARTING_URL**: The URL that the crawler starts from.
- **PERMITTED_DOMAINS**: A list of domain names that the crawler is allowed to visit.
- **BLACKLIST_DOMAINS**: A list of domain names that the crawler is banned from visiting.

### Crawler Settings
- **ROTATE_USER_AGENTS**: A boolean that enables user agent rotation.
- **RESPECT_ROBOTS**: A boolean that enables respecting robots.txt files.
- **FREE_CRAWL**: A boolean that allows the crawler to visit any domain. This will respect the Blacklist.
- **MAX_URLS_TO_VISIT**: The maximum number of URLs that the crawler will visit before stopping.
- **MAX_THREADS**: The maximum number of threads that the crawler will use.
- **CRAWLER_TIMEOUT**: The maximum time the crawler will run.
- **CRAWLER_REQUEST_TIMEOUT**: The maximum time the crawler will wait for a request to return.
- **CRAWLER_REQUEST_DELAY_MS**: The time each crawler thread will wait between visiting sites.

### Data Collection Options
- **COLLECT_HTML**: A boolean that enables the collection of HTML data in db/html
- **COLLECT_IMAGES**: A boolean that enables the collection of image data in db/images

### Logging Options
- **DEBUG**: A boolean that enables debug output.
- **LIVE_LOGGING**: A boolean that will log all URLs as they are visited.

### Database Settings
- **SQLITE_ENABLED**: A boolean that enables pushing results to SQLite.
- **SQLITE_PATH**: The path to the SQLite database file.

## Output
The crawler outputs the URLs of all visited pages to the console.

### SQLite
The crawler collects data from all visited pages in a SQLite database.  
To enable this:
- set `SQLITE_ENABLED` to `true`
- set `SQLITE_PATH` to the path of the SQLite database file

## Implementation
- Starts from a given URL and follows all links to whitelisted domains.  
- Uses a thread pool to visit multiple URLs concurrently.
- Swaps the user agent between requests.
- Respects robots.txt files.
- Supports throttling and timeouts.
- Handles relative paths and redirects.
- Stores selected data in a sqlite database for processing.
