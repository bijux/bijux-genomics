use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use super::test_support;

mod git_authority;
mod score_evidence;

use git_authority::resolve_git_dir;

#[allow(dead_code)]
#[must_use]
pub fn bijux_dna_command(root: &Path) -> Command {
    let executable = bijux_dna_testkit::workspace_support::workspace_binary_path(
        "bijux-dna",
        Path::new(env!("CARGO_BIN_EXE_bijux-dna")),
    )
    .unwrap_or_else(|error| panic!("resolve finalized bijux-dna executable: {error}"));
    let mut command = Command::new(executable);
    command.current_dir(root);
    command
}

#[allow(dead_code)]
pub struct RepoSandbox {
    root: tempfile::TempDir,
    git_dir: PathBuf,
    git_index: Option<PathBuf>,
}

#[allow(dead_code)]
impl RepoSandbox {
    pub fn new(label: &str) -> Result<Self> {
        let source_root = test_support::repo_root()?;
        let git_dir = resolve_git_dir(&source_root)?;
        let sandbox_parent = source_root.join("artifacts/readiness-sandboxes");
        fs::create_dir_all(&sandbox_parent)?;
        let root = tempfile::Builder::new().prefix(label).tempdir_in(sandbox_parent)?;
        let checkout = Command::new("git")
            .current_dir(&source_root)
            .args(["checkout-index", "--all", "--force"])
            .arg(format!("--prefix={}/", root.path().display()))
            .output()
            .map_err(|error| anyhow!("materialize repository sandbox: {error}"))?;
        if !checkout.status.success() {
            return Err(anyhow!(
                "materialize repository sandbox: {}",
                String::from_utf8_lossy(&checkout.stderr).trim()
            ));
        }
        Ok(Self { root, git_dir, git_index: None })
    }

    /// Materialize an owner-selected fixture closure with private Git identity and index.
    pub fn new_selected<F>(label: &str, select: F) -> Result<Self>
    where
        F: FnOnce(&[PathBuf], &dyn Fn(&Path) -> Result<Vec<u8>>) -> Result<Vec<PathBuf>>,
    {
        let source_root = test_support::repo_root()?;
        let sandbox_parent = source_root.join("artifacts/readiness-sandboxes");
        fs::create_dir_all(&sandbox_parent)?;
        let root = tempfile::Builder::new().prefix(label).tempdir_in(sandbox_parent)?;
        let authority = git_authority::materialize_selected(&source_root, root.path(), select)?;
        Ok(Self { root, git_dir: authority.directory, git_index: Some(authority.index) })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    #[must_use]
    pub fn bijux_dna_command(&self) -> Command {
        let mut command = bijux_dna_command(self.path());
        command.envs(self.environment());
        if let Some(index) = &self.git_index {
            command.env("GIT_INDEX_FILE", index).env("GIT_OPTIONAL_LOCKS", "0");
        }
        command
    }

    pub(super) fn environment(&self) -> [(&'static str, PathBuf); 6] {
        [
            ("BIJUX_REPO_ROOT", self.path().to_path_buf()),
            ("BIJUX_BENCHMARK_ROOT", self.path().join("benchmarks")),
            ("ARTIFACT_ROOT", self.path().join("artifacts")),
            ("ISO_ROOT", self.path().join("artifacts")),
            ("GIT_DIR", self.git_dir.clone()),
            ("GIT_WORK_TREE", self.path().to_path_buf()),
        ]
    }
}
