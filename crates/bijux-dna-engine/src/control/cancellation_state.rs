use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::CancellationToken;

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self { cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)) }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}
