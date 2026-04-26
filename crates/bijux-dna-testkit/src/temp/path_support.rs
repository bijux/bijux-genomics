use std::path::{Path, PathBuf};

fn test_tmp_root() -> Option<PathBuf> {
    std::env::var("TEST_TMP_DIR").ok().map(PathBuf::from)
}

fn assert_contained_relative(path: &Path) {
    assert!(path.is_relative(), "test temp paths must be relative: {}", path.display());
    assert!(
        !path.components().any(|component| matches!(component, std::path::Component::ParentDir)),
        "test temp paths must not contain parent traversal: {}",
        path.display()
    );
}

#[must_use]
pub fn temp_path_for(test_name: &str) -> PathBuf {
    super::tempdir_for(test_name).keep()
}

pub fn resolve_under(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    assert_contained_relative(path);
    if let Some(root) = test_tmp_root() {
        return root.join(path);
    }
    std::env::temp_dir().join(path)
}
