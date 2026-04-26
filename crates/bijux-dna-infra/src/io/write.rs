use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::{IoError, IoErrorKind};

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
    if let Some(parent) = non_empty_parent(path) {
        ensure_dir(parent)?;
    }
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

/// Append a single UTF-8 line to a path, creating parent directories if needed.
///
/// # Errors
/// Returns an IO error if the parent cannot be created or the file cannot be opened or written.
pub fn append_line(path: &Path, line: &str) -> Result<(), IoError> {
    if let Some(parent) = non_empty_parent(path) {
        ensure_dir(parent)?;
    }
    let mut file =
        OpenOptions::new().create(true).append(true).open(path).map_err(IoError::from_io)?;
    writeln!(file, "{line}").map_err(IoError::from_io)
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
    let parent = non_empty_parent(path).unwrap_or_else(|| Path::new("."));
    ensure_dir(parent)?;

    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(IoError::from_io)?;
    writer(temp.as_file_mut()).map_err(IoError::from_io)?;
    temp.as_file_mut().sync_all().map_err(IoError::from_io)?;
    apply_target_permissions(path, &mut temp)?;
    temp.persist(path).map_err(|err| IoError::from_io(err.error))?;
    Ok(())
}

#[cfg(unix)]
fn apply_target_permissions(
    path: &Path,
    temp: &mut tempfile::NamedTempFile,
) -> Result<(), IoError> {
    let permissions = existing_or_default_permissions(path);
    temp.as_file_mut().set_permissions(permissions).map_err(IoError::from_io)
}

#[cfg(not(unix))]
fn apply_target_permissions(
    path: &Path,
    temp: &mut tempfile::NamedTempFile,
) -> Result<(), IoError> {
    if let Ok(metadata) = std::fs::metadata(path) {
        temp.as_file_mut().set_permissions(metadata.permissions()).map_err(IoError::from_io)?;
    }
    Ok(())
}

#[cfg(unix)]
fn existing_or_default_permissions(path: &Path) -> std::fs::Permissions {
    if let Ok(metadata) = std::fs::metadata(path) {
        metadata.permissions()
    } else {
        use std::os::unix::fs::PermissionsExt;
        std::fs::Permissions::from_mode(0o644)
    }
}

/// Rename a filesystem path.
///
/// # Errors
/// Returns an IO error if the rename fails.
pub fn rename(src: &Path, dst: &Path) -> Result<(), IoError> {
    if let Some(parent) = non_empty_parent(dst) {
        ensure_dir(parent)?;
    }
    std::fs::rename(src, dst).map_err(IoError::from_io)
}

fn non_empty_parent(path: &Path) -> Option<&Path> {
    path.parent().filter(|parent| !parent.as_os_str().is_empty())
}
