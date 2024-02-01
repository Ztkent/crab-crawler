use reqwest::{Error, Url, header::{self, HeaderValue}};
use rusqlite::Connection;
use std::{path::Path, sync::{Arc, Mutex}};
use rand::seq::SliceRandom;

use crate::{consts, sqlite, tools};

// Fetch HTML from a given URL
pub(crate) fn fetch_html(db_conn: &Arc<Mutex<Connection>>, url: Url) -> Result<String, Error> {
    // Create a new HTTP client
    let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(consts::CRAWLER_REQUEST_TIMEOUT))
    .build()
    .unwrap();

    // Randomly pick a user agent from the list
    let mut user_agent = consts::USER_AGENT_CHROME;
    if consts::ROTATE_USER_AGENTS {
        user_agent = consts::USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
    }

    // Send a GET request to the specified URL and get a response
    let res = client.get(url.clone())
        .header(header::USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
        .send()
        .map_err(|err| {
            err
        })?;
    
    // Get the body of the response as a String
    let body = res.text().map_err(|err| {
        err
    })?;
    
    // Fetch any images from the page
    if consts::COLLECT_HTML {
        match sqlite::insert_html(&db_conn.lock().unwrap(), &tools::format_url_for_storage(url.to_string()), &body.trim().to_string()) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to insert HTML into SQLite: {}", e),
        }
    }

    // Return the body of the response
   Ok(body)
}

// 
pub(crate) fn fetch_image(url: &Url) -> Result<Vec<u8>, reqwest::Error> {
    // Create a new HTTP client
    let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(consts::CRAWLER_REQUEST_TIMEOUT))
    .build()
    .unwrap();

    // Randomly pick a user agent from the list
    let mut user_agent = consts::USER_AGENT_CHROME;
    if consts::ROTATE_USER_AGENTS {
        user_agent = consts::USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
    }

    // Send a GET request to the specified URL and get a response
    let res = client.get(url.clone())
        .header(header::USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
        .send()
        .map_err(|err| {
            err
        })?;

    // Check if the Content-Type is an image
    let content_type = res.headers().get(header::CONTENT_TYPE);
    if let Some(content_type) = content_type {
        if !content_type.to_str().unwrap().starts_with("image/") {
            tools::debug_log(&format!("The body of the response is not an image: {}", url));
            return Ok(Vec::new());
        }
    }

    // Get the body of the response as bytes
    let bytes = res.bytes().map_err(|err| {
        eprintln!("Failed to read image data: {}", err);
        err
    })?;

    Ok(bytes.to_vec())
}

// Handle any relative paths that we've encountered.
pub(crate) fn handle_relative_paths(url: &str, referrer_url: &String) -> Result<String, (Option<Url>, bool)> {
    let mut formatted_url = url.trim().to_string();
    if formatted_url.starts_with("www") || formatted_url.starts_with("http") {
        // This is a valid URL
        return Ok(formatted_url);
    } else if let Some(index) = url.find("#") {
        // Remove any anchors from the URL
        formatted_url = formatted_url[..index].trim().to_string();
    } 
    
     // Skip any empty URLs
    if formatted_url == "" || formatted_url == "/" || formatted_url == "#" || formatted_url.starts_with("?") || formatted_url == "\\\"" || formatted_url == "..//"{
        return Err((None, false));
    }

    // Handle any relative paths
    if formatted_url.starts_with("mailto") || formatted_url.starts_with("whatsapp") || formatted_url.starts_with("fb-messenger") || 
        formatted_url.starts_with("tel") || formatted_url.starts_with("sms") || formatted_url.starts_with("facetime") || 
        formatted_url.starts_with("skype") || formatted_url.starts_with("slack") || formatted_url.starts_with("zoom") {
        return Err((None, false));
    } else if formatted_url.starts_with("itms") || formatted_url.starts_with("market") { 
        // Apple App Store or Google Play Store
        return Err((None, false));
    } else if formatted_url.starts_with("javascript") {
        return Err((None, false));
    } else if formatted_url.contains(":invalid") {
        return Err((None, false));
    } else if formatted_url.starts_with("data:image") {
        // Data URL, such as a base64 image path. Maybe these are worth your time.
        return Err((None, false));
    } else if formatted_url.starts_with("clkn/http/") {
        // This is a redirect URL from Google Ads.
        formatted_url = format!("http://{}", formatted_url.trim_start_matches("clkn/http/"));
    } else if formatted_url.starts_with("clkn/rel/") {
        // This is a redirect URL from Google Ads.
        // Relative path to a url. such as "/politics/congress".
        let ref_url = Url::parse(referrer_url);
        if ref_url.is_err() {
            tools::debug_log(&format!("Invalid referrer URL: {}", referrer_url));
            return Err((None, false));
        }
        let ref_url = ref_url.unwrap();
        let ref_domain = ref_url.domain().unwrap_or("").to_string();
        formatted_url = format!("{}{}", ref_domain, formatted_url.trim_start_matches("clkn/rel/"));
    } else if formatted_url.starts_with("//") {
        // Protocol-relative URL. such as "//www.cnn.com".
        formatted_url = format!("https:{}", formatted_url);
    } else if formatted_url.starts_with("/") {
        // Relative path to a url. such as "/politics/congress".
        let ref_url = Url::parse(referrer_url);
        if ref_url.is_err() {
            tools::debug_log(&format!("Invalid referrer URL: {}", referrer_url));
            return Err((None, false));
        }
        let ref_url = ref_url.unwrap();
        let ref_domain = ref_url.domain().unwrap_or("").to_string();
        formatted_url = format!("{}{}", ref_domain, formatted_url);
    } else if formatted_url.starts_with("../") || formatted_url.starts_with("./../"){
        // is ./../ the same as ../
        if formatted_url.starts_with("./") {
            formatted_url = formatted_url[2..].to_string();
        }
        // Relative path to a url. such as "../politics/congress".
        let ref_url = Url::parse(referrer_url).unwrap();
        let mut path = Path::new(ref_url.path());
        while formatted_url.starts_with("../") {
            formatted_url = formatted_url[3..].to_string();
            if let Some(parent) = path.parent() {
                path = parent;
            }
        }
        let mut mutable_ref_url = ref_url.clone();
        mutable_ref_url.set_path(path.to_str().unwrap());
        // Handle the slash at the end of the path
        if !mutable_ref_url.as_str().ends_with("/") && !formatted_url.starts_with("/") {
            mutable_ref_url.set_path(format!("{}/", mutable_ref_url.path()).as_str());
        }
        formatted_url = format!("{}{}", mutable_ref_url, formatted_url.trim_start_matches(".."));
    } else if formatted_url.starts_with("./") {
        // Another folder up, such as "./politics/congress".
        let ref_url = Url::parse(referrer_url).unwrap();
        let mut path = Path::new(ref_url.path());
        if let Some(parent) = path.parent() {
            path = parent;
        }
        let mut mutable_ref_url = ref_url.clone();
        mutable_ref_url.set_path(path.to_str().unwrap());
        if !mutable_ref_url.as_str().ends_with("/") && !formatted_url.starts_with("/") {
            mutable_ref_url.set_path(format!("{}/", mutable_ref_url.path()).as_str());
        }
        formatted_url = format!("{}{}", mutable_ref_url, formatted_url.trim_start_matches("./"));
    } else {
        // Likely a relative path to a url. such as "politics/congress.html".
        let ref_url = Url::parse(referrer_url).unwrap();
        let mut mutable_ref_url = ref_url.clone();
        if !mutable_ref_url.as_str().ends_with("/") {
            let mut path = Path::new(ref_url.path());
            if let Some(parent) = path.parent() {
                path = parent;
            }
            mutable_ref_url.set_path(path.to_str().unwrap());
        }
        if !mutable_ref_url.as_str().ends_with("/") && !formatted_url.starts_with("/") {
            mutable_ref_url.set_path(format!("{}/", mutable_ref_url.path()).as_str());
        }
        formatted_url = format!("{}{}", mutable_ref_url, formatted_url);
    }

    if consts::LOG_RELATIVE_PATHS {
        if formatted_url != url {
            tools::debug_log(&format!("Formatted Relative URL [{}] to [{}] from [{}]", url, formatted_url, referrer_url));
        }
    }
    Ok(formatted_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_fetch_html() {
        let url = Url::parse("https://www.cnn.com").unwrap();
        let db_conn = match sqlite::connect_sqlite_inmemory() {
            Ok(connection) => connection.unwrap(),
            Err(e) => {
                eprintln!("Failed to connect to SQLite and migrate: {}", e);
                return;
            }
        };
        let db_conn: Arc<Mutex<Connection>> = Arc::new(Mutex::new(db_conn));
        let result = fetch_html(&db_conn, url);
        assert!(result.is_ok());
    }
}