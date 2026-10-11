use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};

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

    /// Admit a local output without creating it or following a link outside artifacts.
    ///
    /// # Errors
    /// Rejects foreign roots, parent traversal, inaccessible ancestors, and escape links.
    pub fn admit_artifact_path(&self, declared: &Path) -> Result<PathBuf> {
        ensure!(
            !declared.components().any(|part| part == Component::ParentDir),
            "artifact path must not contain parent traversal"
        );
        let candidate =
            if declared.is_absolute() { declared.to_path_buf() } else { self.root.join(declared) };
        let boundary = self.root.join("artifacts");
        ensure!(candidate.starts_with(&boundary), "output must stay under workspace artifacts/");
        match boundary.symlink_metadata() {
            Ok(metadata) => ensure!(metadata.is_dir(), "artifact root must be a regular directory"),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("inspect artifact root"),
        }
        let resolved_boundary = match boundary.canonicalize() {
            Ok(path) => path,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.root.canonicalize().context("resolve workspace root")?.join("artifacts")
            }
            Err(error) => return Err(error).context("resolve artifact root"),
        };
        // A future output inherits the boundary of its nearest existing ancestor.
        for ancestor in candidate.ancestors() {
            match ancestor.symlink_metadata() {
                Ok(_) => {
                    let resolved = ancestor.canonicalize().with_context(|| {
                        format!("resolve output ancestor {}", ancestor.display())
                    })?;
                    if ancestor.starts_with(&boundary) {
                        ensure!(
                            resolved.starts_with(&resolved_boundary),
                            "artifact path escapes through a symlink"
                        );
                    }
                    return Ok(candidate);
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("inspect output destination"),
            }
        }
        anyhow::bail!("artifact path has no resolvable filesystem ancestor")
    }
}

#[cfg(test)]
mod artifact_path_tests {
    use super::*;

    #[test]
    fn admits_only_the_workspace_artifact_root_and_its_children() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        for path in ["artifacts", "artifacts/ci/future", "./artifacts/ci"] {
            assert!(workspace.admit_artifact_path(Path::new(path)).is_ok(), "{path}");
        }
        assert_eq!(
            workspace.admit_artifact_path(&workspace.path("artifacts"))?,
            workspace.path("artifacts")
        );
        for path in ["", "artifacts-elsewhere", "other/artifacts", "artifacts/../artifacts"] {
            assert!(workspace.admit_artifact_path(Path::new(path)).is_err(), "{path}");
        }
        let other = tempfile::tempdir()?;
        assert!(workspace.admit_artifact_path(&other.path().join("artifacts")).is_err());
        assert!(!workspace.path("artifacts").exists(), "admission must not create outputs");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_an_artifact_root_symlink_without_mutating_it() -> Result<()> {
        let root = tempfile::tempdir()?;
        let other = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        std::os::unix::fs::symlink(other.path(), workspace.path("artifacts"))?;
        assert!(workspace.admit_artifact_path(Path::new("artifacts/output")).is_err());
        assert!(workspace.path("artifacts").is_symlink());
        Ok(())
    }
}
