mod error;
mod read;
mod remove;
mod stable_surface;
mod write;

use crate::{retry_with, RetryPolicy, SystemClock};

pub use stable_surface::*;

/// Atomically write bytes with retry/backoff.
///
/// # Errors
/// Returns the last IO error after exhausting retries.
pub fn atomic_write_bytes_with_retry(
    path: &std::path::Path,
    bytes: &[u8],
    policy: &RetryPolicy,
) -> Result<(), IoError> {
    retry_with(policy, &SystemClock, |_| atomic_write_bytes(path, bytes))
}
