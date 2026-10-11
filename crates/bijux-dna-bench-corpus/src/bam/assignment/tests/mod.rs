#![cfg(test)]
mod admission;
mod identity;
mod sources;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("corpus crate must reside under workspace crates")
}
