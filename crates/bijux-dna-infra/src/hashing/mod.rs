use std::path::Path;

use crate::IoError;

mod file_digest;

/// Hash a file using SHA-256.
///
/// # Errors
/// Returns an IO error if the file cannot be read.
pub fn hash_file_sha256(path: &Path) -> Result<String, IoError> {
    file_digest::hash_file_sha256(path)
}
