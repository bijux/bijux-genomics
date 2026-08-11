use std::path::Path;

use crate::IoError;
use sha2::{Digest, Sha256};

mod file_digest;
mod path_digest;

/// Hash a file using SHA-256.
///
/// # Errors
/// Returns an IO error if the file cannot be read.
pub fn hash_file_sha256(path: &Path) -> Result<String, IoError> {
    file_digest::hash_file_sha256(path)
}

/// Hash a regular file or a directory tree using SHA-256.
///
/// File identities are byte-compatible with [`hash_file_sha256`]. Directory identities include
/// every relative path, entry kind, file digest, and un-followed symbolic-link target in lexical
/// path order.
///
/// # Errors
/// Returns an IO error when the path is missing, has an unsupported type, or cannot be read.
pub fn hash_path_sha256(path: &Path) -> Result<String, IoError> {
    path_digest::hash_path_sha256(path)
}

/// Hash an in-memory payload using SHA-256.
#[must_use]
pub fn hash_bytes_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    sha256_hex(digest)
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let digest = digest.as_ref();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
