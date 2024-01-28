use crate::constants as consts;

pub(crate) fn debug_log(log_message: &str) {
    if consts::DEBUG {
        eprintln!("{}", log_message);
    }
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