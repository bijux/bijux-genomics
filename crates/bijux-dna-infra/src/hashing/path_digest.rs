use std::path::Path;

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::{IoError, IoErrorKind};

pub(super) fn hash_path_sha256(path: &Path) -> Result<String, IoError> {
    if path.is_file() {
        return super::file_digest::hash_file_sha256(path);
    }
    if path.is_dir() {
        return hash_directory(path);
    }
    Err(IoError::new(
        IoErrorKind::Corruption,
        format!("unsupported hash input path type: {}", path.display()),
    ))
}

fn hash_directory(root: &Path) -> Result<String, IoError> {
    let mut entries = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(walk_error)?;
    entries.sort_by(|left, right| left.path().cmp(right.path()));

    let mut hasher = Sha256::new();
    for entry in entries {
        let path = entry.path();
        if path == root {
            continue;
        }
        let relative = path.strip_prefix(root).map_err(|error| {
            IoError::new(
                IoErrorKind::Corruption,
                format!("strip directory hash prefix for {}: {error}", path.display()),
            )
        })?;
        hasher.update(relative.to_string_lossy().as_bytes());
        if entry.file_type().is_dir() {
            hasher.update(b"\0dir\0");
            continue;
        }
        if entry.file_type().is_symlink() {
            hasher.update(b"\0symlink\0");
            let target = std::fs::read_link(path).map_err(IoError::from_io)?;
            hasher.update(target.to_string_lossy().as_bytes());
            continue;
        }
        hasher.update(b"\0file\0");
        hasher.update(super::file_digest::hash_file_sha256(path)?.as_bytes());
    }
    Ok(super::sha256_hex(hasher.finalize()))
}

fn walk_error(error: walkdir::Error) -> IoError {
    let message = error.to_string();
    error
        .into_io_error()
        .map_or_else(|| IoError::new(IoErrorKind::Corruption, message), IoError::from_io)
}
