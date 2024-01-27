// tests/crawl_tests.rs
use crate::crawl::fetch_html;
use url::Url;

#[test]
fn test_fetch_html() {
    let url = Url::parse("https://www.example.com").unwrap();
    let result = fetch_html(url);
    assert!(result.is_ok());
}