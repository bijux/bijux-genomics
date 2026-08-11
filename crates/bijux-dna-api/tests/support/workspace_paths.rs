#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

static REPO_SANDBOX: OnceLock<std::result::Result<PathBuf, String>> = OnceLock::new();

fn looks_like_repo_root(path: &Path) -> bool {
    path.join("Cargo.lock").is_file()
        && path.join("crates").is_dir()
        && path.join("configs").is_dir()
}

/// Resolve an isolated workspace root for repository-writing contract tests.
///
/// # Errors
///
/// Returns an error when the source repository cannot be resolved or a tracked
/// checkout cannot be materialized for the contract test process.
pub fn repo_root() -> Result<PathBuf> {
    match REPO_SANDBOX.get_or_init(|| {
        let source_root = source_repo_root().map_err(|error| format!("{error:#}"))?;
        let sandbox_parent = source_root.join("artifacts/api-contract-sandboxes");
        fs::create_dir_all(&sandbox_parent)
            .map_err(|error| {
                anyhow!("create API contract sandbox root {}: {error}", sandbox_parent.display())
            })
            .map_err(|error| format!("{error:#}"))?;
        let root = tempfile::Builder::new()
            .prefix("api-contract-")
            .tempdir_in(&sandbox_parent)
            .map_err(|error| format!("create API contract sandbox: {error}"))?;
        let checkout = Command::new("git")
            .current_dir(&source_root)
            .args(["checkout-index", "--all", "--force"])
            .arg(format!("--prefix={}/", root.path().display()))
            .output()
            .map_err(|error| format!("materialize API contract sandbox: {error}"))?;
        if !checkout.status.success() {
            return Err(format!(
                "materialize API contract sandbox: {}",
                String::from_utf8_lossy(&checkout.stderr).trim()
            ));
        }

        Ok(root.keep())
    }) {
        Ok(path) => Ok(path.clone()),
        Err(error) => Err(anyhow!(error.clone())),
    }
}

fn source_repo_root() -> Result<PathBuf> {
    if let Some(path) =
        std::env::var_os("BIJUX_REPO_ROOT").filter(|value| !value.is_empty()).map(PathBuf::from)
    {
        if looks_like_repo_root(&path) {
            return Ok(path);
        }
        return Err(anyhow!(
            "BIJUX_REPO_ROOT does not point to a bijux repository root: {}",
            path.display()
        ));
    }

    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve current directory: {err}"))?;
    for candidate in cwd.ancestors() {
        if looks_like_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(anyhow!("unable to resolve repository root from {}", cwd.display()))
}

/// Resolve the crate root for repository-scoped fixtures and snapshots.
///
/// # Errors
///
/// Propagates any repository root resolution failure.
pub fn crate_root(crate_name: &str) -> Result<PathBuf> {
    Ok(source_repo_root()?.join("crates").join(crate_name))
}

/// Resolve the `src` directory for a crate under test.
///
/// # Errors
///
/// Propagates any repository root resolution failure.
pub fn crate_src(crate_name: &str) -> Result<PathBuf> {
    Ok(crate_root(crate_name)?.join("src"))
}

/// Resolve the snapshot directory for a crate under test.
///
/// # Errors
///
/// Propagates any repository root resolution failure.
pub fn crate_snapshots(crate_name: &str) -> Result<PathBuf> {
    Ok(crate_root(crate_name)?.join("tests").join("snapshots"))
}

const TEST_LOCK_ROOT: &str = "artifacts/test-locks";

pub struct RepoProcessLock {
    _file_lock: bijux_dna_infra::FileLock,
}

impl RepoProcessLock {
    pub fn acquire(name: &str) -> Result<Self> {
        let repo_root = source_repo_root()?;
        let path = repo_root.join(TEST_LOCK_ROOT).join(format!("{name}.lock"));
        let file_lock = bijux_dna_infra::FileLock::try_acquire(&path)
            .map_err(|error| anyhow!("acquire repo test lock `{}`: {error}", path.display()))?;
        Ok(Self { _file_lock: file_lock })
    }
}

pub fn bench_output_lock() -> Result<RepoProcessLock> {
    RepoProcessLock::acquire("bijux-dna-api-bench-output")
}
