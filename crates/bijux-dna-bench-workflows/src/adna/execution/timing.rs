//! Owner: bijux-dna-bench-workflows
//! Ancient DNA execution timing.
use super::super::{SystemTime, UNIX_EPOCH};

pub(crate) fn timestamp_marker() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "0".to_string(), |duration| duration.as_secs().to_string())
}
