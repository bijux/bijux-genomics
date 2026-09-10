//! Owner: bijux-dna-bench-population
//! Population structure execution timing.
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}
