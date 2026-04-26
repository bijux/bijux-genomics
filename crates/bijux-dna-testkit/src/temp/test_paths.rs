use std::path::{Path, PathBuf};

fn assert_contained_relative(path: &Path) {
    assert!(path.is_relative(), "test path children must be relative: {}", path.display());
    assert!(
        !path.components().any(|component| matches!(component, std::path::Component::ParentDir)),
        "test path children must not contain parent traversal: {}",
        path.display()
    );
}

#[derive(Debug, Clone)]
pub struct TestPaths {
    root: PathBuf,
}

impl TestPaths {
    #[must_use]
    pub fn new(test_name: &str) -> Self {
        let dir = super::tempdir_for(test_name);
        let root = dir.keep();
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn child(&self, rel: impl AsRef<Path>) -> PathBuf {
        let rel = rel.as_ref();
        assert_contained_relative(rel);
        self.root.join(rel)
    }
}
