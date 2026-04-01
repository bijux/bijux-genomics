use std::path::Path;

use super::IoError;

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
