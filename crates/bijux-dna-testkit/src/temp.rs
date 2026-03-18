use std::path::{Path, PathBuf};

use tempfile::TempDir;

fn test_tmp_root() -> Option<PathBuf> {
    std::env::var("TEST_TMP_DIR").ok().map(PathBuf::from)
}

/// Create a test temp directory rooted under `TEST_TMP_DIR` when available.
///
/// # Panics
/// Panics if the temporary directory cannot be created.
#[must_use]
pub fn tempdir_for(test_name: &str) -> TempDir {
    let prefix = format!("bijux-dna-{test_name}-");
    if let Some(root) = test_tmp_root() {
        if root.exists() {
            return tempfile::Builder::new()
                .prefix(&prefix)
                .tempdir_in(&root)
                .unwrap_or_else(|err| panic!("tempdir_in {}: {err}", root.display()));
        }
    }
    tempfile::Builder::new()
        .prefix(&prefix)
        .tempdir()
        .unwrap_or_else(|err| panic!("tempdir: {err}"))
}

#[must_use]
pub fn temp_path_for(test_name: &str) -> PathBuf {
    tempdir_for(test_name).keep()
}

pub fn resolve_under(path: impl AsRef<Path>) -> PathBuf {
    if let Some(root) = test_tmp_root() {
        return root.join(path);
    }
    std::env::temp_dir().join(path)
}

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

#[derive(Debug, Clone)]
pub struct TestPaths {
    root: PathBuf,
}

impl TestPaths {
    #[must_use]
    pub fn new(test_name: &str) -> Self {
        let dir = tempdir_for(test_name);
        let root = dir.keep();
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn child(&self, rel: impl AsRef<Path>) -> PathBuf {
        self.root.join(rel)
    }
}
