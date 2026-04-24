use std::path::Path;

use crate::IoError;

/// Create a managed temporary directory.
///
/// # Errors
/// Returns an IO error if the temp directory cannot be created.
pub fn temp_dir(prefix: &str) -> Result<tempfile::TempDir, IoError> {
    tempfile::Builder::new().prefix(prefix).tempdir().map_err(IoError::from_io)
}

/// Create a managed temporary directory under a base path.
///
/// # Errors
/// Returns an IO error if the temp directory cannot be created.
pub fn temp_dir_in(base: &Path, prefix: &str) -> Result<tempfile::TempDir, IoError> {
    tempfile::Builder::new().prefix(prefix).tempdir_in(base).map_err(IoError::from_io)
}
