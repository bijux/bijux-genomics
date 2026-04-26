use std::path::{Path, PathBuf};

use crate::{ensure_dir, IoError};

/// Create a managed temporary directory.
///
/// # Errors
/// Returns an IO error if the temp directory cannot be created.
pub fn temp_dir(prefix: &str) -> Result<tempfile::TempDir, IoError> {
    if let Some(base) = test_tmp_dir() {
        return temp_dir_in(&base, prefix);
    }
    tempfile::Builder::new().prefix(prefix).tempdir().map_err(IoError::from_io)
}

/// Create a managed temporary directory under a base path.
///
/// # Errors
/// Returns an IO error if the temp directory cannot be created.
pub fn temp_dir_in(base: &Path, prefix: &str) -> Result<tempfile::TempDir, IoError> {
    ensure_dir(base)?;
    tempfile::Builder::new().prefix(prefix).tempdir_in(base).map_err(IoError::from_io)
}

fn test_tmp_dir() -> Option<PathBuf> {
    std::env::var_os("TEST_TMP_DIR").filter(|value| !value.is_empty()).map(PathBuf::from)
}
