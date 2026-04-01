use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CancellationToken {
    pub(super) cancelled: Arc<std::sync::atomic::AtomicBool>,
}
