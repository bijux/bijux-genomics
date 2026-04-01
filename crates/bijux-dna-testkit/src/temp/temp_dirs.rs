use std::path::PathBuf;

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
