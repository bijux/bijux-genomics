//! Owner: bijux-dna-bench-population
//! Demography execution timing.
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) fn timestamp_marker() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
    format!("{}", now.as_secs())
}
