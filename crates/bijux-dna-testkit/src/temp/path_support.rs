use std::path::{Path, PathBuf};

fn test_tmp_root() -> Option<PathBuf> {
    std::env::var("TEST_TMP_DIR").ok().map(PathBuf::from)
}

#[must_use]
pub fn temp_path_for(test_name: &str) -> PathBuf {
    super::tempdir_for(test_name).keep()
}

pub fn resolve_under(path: impl AsRef<Path>) -> PathBuf {
    if let Some(root) = test_tmp_root() {
        return root.join(path);
    }
    std::env::temp_dir().join(path)
}
