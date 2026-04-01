mod env_override;
mod root_detection;

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use env_override::declared_repo_root;
use root_detection::looks_like_repo_root;

pub(crate) fn resolve_repo_root() -> Result<PathBuf> {
    if let Some(path) = declared_repo_root() {
        return Ok(path);
    }

    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve current dir: {err}"))?;
    for candidate in cwd.ancestors() {
        if looks_like_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(anyhow!(
        "BIJUX_REPO_ROOT must be declared when runner execution needs repository manifests"
    ))
}
