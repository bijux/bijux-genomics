use std::path::{Path, PathBuf};

#[must_use]
pub fn sorted_read_dir_paths(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read_dir failed: {err}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    out.sort();
    out
}
