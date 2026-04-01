use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    /// # Errors
    /// Returns an error if the current workspace root cannot be determined.
    pub fn resolve() -> Result<Self> {
        let root = super::workspace_root::resolve_workspace_root()?;
        Ok(Self { root })
    }

    #[must_use]
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    #[must_use]
    pub fn rel<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}
