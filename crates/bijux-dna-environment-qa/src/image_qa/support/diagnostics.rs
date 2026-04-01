pub struct StdoutLogger {
    debug: bool,
}

impl StdoutLogger {
    #[must_use]
    pub fn new() -> Self {
        Self { debug: false }
    }

    #[allow(clippy::unused_self)]
    pub fn info(&self, message: &str) {
        println!("{message}");
    }

    #[allow(clippy::unused_self)]
    pub fn debug(&self, message: &str) {
        if self.debug {
            println!("{message}");
        }
    }
}

#[must_use]
pub fn trace_enabled() -> bool {
    std::env::var("BIJUX_TRACE_ENGINE").is_ok()
}
