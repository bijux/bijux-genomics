mod error;
mod write;

use std::path::Path;

use crate::{retry_with, RetryPolicy, SystemClock};

pub use error::{classify_io_error, IoError, IoErrorKind};
pub use write::{
    atomic_write_bytes, atomic_write_json, atomic_write_with, create_file, ensure_dir, rename,
    write_bytes, write_string,
};

/// Atomically write bytes with retry/backoff.
///
/// # Errors
/// Returns the last IO error after exhausting retries.
pub fn atomic_write_bytes_with_retry(
    path: &Path,
    bytes: &[u8],
    policy: &RetryPolicy,
) -> Result<(), IoError> {
    retry_with(policy, &SystemClock, |_| atomic_write_bytes(path, bytes))
}

/// Read a file with a maximum byte limit.
///
/// # Errors
/// Returns an IO error if reading fails or the file exceeds the limit.
pub fn read_to_end_bounded(path: &Path, max_bytes: usize) -> Result<Vec<u8>, IoError> {
    let file = File::open(path).map_err(IoError::from_io)?;
    let mut buffer = Vec::new();
    let limit = match u64::try_from(max_bytes) {
        Ok(value) => value.saturating_add(1),
        Err(_) => u64::MAX,
    };
    file.take(limit)
        .read_to_end(&mut buffer)
        .map_err(IoError::from_io)?;
    if buffer.len() > max_bytes {
        return Err(IoError::new(
            IoErrorKind::Corruption,
            format!("file exceeds max bytes ({max_bytes})"),
        ));
    }
    Ok(buffer)
}

/// Remove a file.
///
/// # Errors
/// Returns an IO error if the removal fails.
pub fn remove_file(path: &Path) -> Result<(), IoError> {
    std::fs::remove_file(path).map_err(IoError::from_io)
}

/// Remove a directory and all contents.
///
/// # Errors
/// Returns an IO error if removal fails.
pub fn remove_dir_all(path: &Path) -> Result<(), IoError> {
    std::fs::remove_dir_all(path).map_err(IoError::from_io)
}

/// Remove a file or directory if it exists.
///
/// # Errors
/// Returns an IO error if removal fails.
pub fn remove_path_if_exists(path: &Path) -> Result<(), IoError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(IoError::from_io(err)),
    };
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        remove_dir_all(path)
    } else {
        remove_file_if_exists(path)
    }
}

/// Remove a file if it exists.
///
/// # Errors
/// Returns an IO error for failures other than missing files.
pub fn remove_file_if_exists(path: &Path) -> Result<(), IoError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(IoError::from_io(err)),
    }
}
