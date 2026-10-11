use std::path::Path;

use crate::IoError;
use sha2::{Digest, Sha256};

mod file_digest;

/// Hash a file using SHA-256.
///
/// # Errors
/// Returns an IO error if the file cannot be read.
pub fn hash_file_sha256(path: &Path) -> Result<String, IoError> {
    file_digest::hash_file_sha256(path)
}

/// Hash an in-memory payload using SHA-256.
#[must_use]
pub fn hash_bytes_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
