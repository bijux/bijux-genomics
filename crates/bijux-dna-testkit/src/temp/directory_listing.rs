use std::path::{Path, PathBuf};

/// Return sorted child paths for a directory.
///
/// # Panics
/// Panics if `dir` cannot be read.
#[must_use]
pub fn sorted_read_dir_paths(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let dir = dir.as_ref();
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read_dir {} failed: {err}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    out.sort();
    out
}
