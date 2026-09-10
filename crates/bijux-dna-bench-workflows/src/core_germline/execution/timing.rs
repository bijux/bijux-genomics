//! Owner: bijux-dna-bench-workflows
//! Core germline execution timing.
use super::super::{SystemTime, UNIX_EPOCH};

pub(crate) fn timestamp_marker() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
        .to_string()
}
