use std::path::{Path, PathBuf};

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
        self.root.join(rel)
    }
}
