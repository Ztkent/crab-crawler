use reqwest::Url;
use robotstxt::DefaultMatcher;
use crate::constants as consts;

pub(crate) fn debug_log(log_message: &str) {
    if consts::DEBUG {
        eprintln!("{}", log_message);
    }
}

pub(crate) fn is_robots_txt_blocked(url: Url) -> bool {
    // Todo: we have to cache this or its death for performance
    let robots_url = format!("https://{}/robots.txt", url.domain().unwrap());
    let robots_txt = match reqwest::blocking::get(&robots_url) {
        Ok(response) => response.text().unwrap(),
        Err(_) => return false,
    };

    let mut matcher = DefaultMatcher::default();
    let allowed = matcher.allowed_by_robots(&robots_txt, consts::USER_AGENTS.into_iter().collect(), url.as_str());
    !allowed
}

// Defer is a helper struct that allows us to run a function when the struct is dropped.
// Using this similar to defer in Go, we can ensure that a function is run when the current scope is exited.
pub(crate) struct Defer<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> Defer<F> {
    pub(crate) fn new(f: F) -> Defer<F> {
        Defer { f: Some(f) }
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}