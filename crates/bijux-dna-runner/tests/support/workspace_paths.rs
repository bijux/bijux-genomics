#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

fn looks_like_repo_root(path: &Path) -> bool {
    path.join("Cargo.lock").is_file()
        && path.join("crates").is_dir()
        && path.join("configs").is_dir()
}

/// Resolve the repository root that contains this crate's workspace.
///
/// # Errors
/// Returns an error when the current working directory is outside the repository tree.
pub fn repo_root() -> Result<PathBuf> {
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

/// Resolve the root directory for a workspace crate.
///
/// # Errors
/// Returns an error when the workspace root cannot be resolved.
pub fn crate_root(crate_name: &str) -> Result<PathBuf> {
    Ok(repo_root()?.join("crates").join(crate_name))
}

/// Resolve the `src` directory for a workspace crate.
///
/// # Errors
/// Returns an error when the workspace root cannot be resolved.
pub fn crate_src(crate_name: &str) -> Result<PathBuf> {
    Ok(crate_root(crate_name)?.join("src"))
}
