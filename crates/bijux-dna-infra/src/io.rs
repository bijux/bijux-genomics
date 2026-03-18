use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use thiserror::Error;

use crate::{retry_with, RetryPolicy, SystemClock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoErrorKind {
    Permission,
    Missing,
    Transient,
    Corruption,
    LockTimeout,
    Other,
}

#[derive(Debug, Error)]
#[error("{kind:?}: {message}")]
pub struct IoError {
    pub kind: IoErrorKind,
    pub message: String,
    #[source]
    pub source: Option<std::io::Error>,
}

impl IoError {
    #[must_use]
    pub fn from_io(err: std::io::Error) -> Self {
        let kind = classify_io_error(&err);
        Self {
            kind,
            message: err.to_string(),
            source: Some(err),
        }
    }

    #[must_use]
    pub fn new(kind: IoErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }
}

#[must_use]
pub fn classify_io_error(err: &std::io::Error) -> IoErrorKind {
    use std::io::ErrorKind;

    match err.kind() {
        ErrorKind::NotFound => IoErrorKind::Missing,
        ErrorKind::PermissionDenied => IoErrorKind::Permission,
        ErrorKind::TimedOut | ErrorKind::WouldBlock | ErrorKind::Interrupted => {
            IoErrorKind::Transient
        }
        ErrorKind::InvalidData | ErrorKind::InvalidInput | ErrorKind::UnexpectedEof => {
            IoErrorKind::Corruption
        }
        _ => IoErrorKind::Other,
    }
}

/// Ensure a directory exists, creating it if needed.
///
/// # Errors
/// Returns an IO error if the directory cannot be created.
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<(), IoError> {
    std::fs::create_dir_all(path.as_ref()).map_err(IoError::from_io)
}

/// Create a file for writing, ensuring the parent directory exists first.
///
/// # Errors
/// Returns an IO error if the parent cannot be created or the file cannot be opened.
pub fn create_file(path: &Path) -> Result<File, IoError> {
    let parent = path
        .parent()
        .ok_or_else(|| IoError::new(IoErrorKind::Missing, "path has no parent"))?;
    ensure_dir(parent)?;
    File::create(path).map_err(IoError::from_io)
}

/// Atomically write bytes to a path (temp + rename).
///
/// # Errors
/// Returns an IO error if the write or rename fails.
pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<(), IoError> {
    atomic_write_with(path, |file| file.write_all(bytes))
}

/// Write bytes to a path with the standard atomic write policy.
///
/// # Errors
/// Returns an IO error if serialization or writing fails.
pub fn write_bytes<P: AsRef<Path>, B: AsRef<[u8]>>(path: P, bytes: B) -> Result<(), IoError> {
    atomic_write_bytes(path.as_ref(), bytes.as_ref())
}

/// Write a UTF-8 string to a path with the standard atomic write policy.
///
/// # Errors
/// Returns an IO error if writing fails.
pub fn write_string<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), IoError> {
    write_bytes(path, contents.as_bytes())
}

/// Atomically write JSON to a path (temp + rename).
///
/// # Errors
/// Returns an IO error if serialization or writing fails.
pub fn atomic_write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), IoError> {
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("serialize json: {err}")))?;
    atomic_write_bytes(path, &payload)
}

/// Atomically write using a custom writer function.
///
/// # Errors
/// Returns an IO error if the write or rename fails.
pub fn atomic_write_with<F>(path: &Path, writer: F) -> Result<(), IoError>
where
    F: FnOnce(&mut File) -> std::io::Result<()>,
{
    let parent = path
        .parent()
        .ok_or_else(|| IoError::new(IoErrorKind::Missing, "path has no parent"))?;
    ensure_dir(parent)?;

    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(IoError::from_io)?;
    writer(temp.as_file_mut()).map_err(IoError::from_io)?;
    temp.as_file_mut().sync_all().map_err(IoError::from_io)?;
    #[cfg(unix)]
    let perm = {
        use std::os::unix::fs::PermissionsExt;
        Some(std::fs::Permissions::from_mode(0o644))
    };
    #[cfg(not(unix))]
    let perm: Option<std::fs::Permissions> = None;
    if let Some(perm) = perm {
        temp.as_file_mut()
            .set_permissions(perm)
            .map_err(IoError::from_io)?;
    }
    temp.persist(path)
        .map_err(|err| IoError::from_io(err.error))?;
    Ok(())
}

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
    let mut file = File::open(path).map_err(IoError::from_io)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(IoError::from_io)?;
    if buffer.len() > max_bytes {
        return Err(IoError::new(
            IoErrorKind::Corruption,
            format!("file exceeds max bytes ({max_bytes})"),
        ));
    }
    Ok(buffer)
}

/// Rename a filesystem path.
///
/// # Errors
/// Returns an IO error if the rename fails.
pub fn rename(src: &Path, dst: &Path) -> Result<(), IoError> {
    std::fs::rename(src, dst).map_err(IoError::from_io)
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
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        remove_dir_all(path)
    } else {
        remove_file(path)
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
