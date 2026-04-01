use std::path::PathBuf;

use anyhow::{Context, Result};

/// # Errors
/// Returns an error if the repository root cannot be resolved from the crate manifest path.
pub(super) fn resolve_workspace_root() -> Result<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("bijux-dna-dev manifest has no parent")?
        .parent()
        .context("workspace root is not two levels above crate manifest")?
        .to_path_buf();
    Ok(root)
}
