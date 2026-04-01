mod error;
mod read;
mod remove;
mod write;

use crate::{retry_with, RetryPolicy, SystemClock};

pub use error::{classify_io_error, IoError, IoErrorKind};
pub use read::read_to_end_bounded;
pub use remove::{remove_dir_all, remove_file, remove_file_if_exists, remove_path_if_exists};
pub use write::{
    atomic_write_bytes, atomic_write_json, atomic_write_with, create_file, ensure_dir, rename,
    write_bytes, write_string,
};

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
