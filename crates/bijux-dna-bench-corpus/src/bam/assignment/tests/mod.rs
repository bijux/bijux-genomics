#![cfg(test)]
mod admission;
mod identity;
mod sources;

use anyhow::{Context, Result};
use std::path::PathBuf;
fn repo_root() -> Result<PathBuf> {
    let root = runtime_repository::repo_root();
    root.canonicalize()
        .with_context(|| format!("resolve corpus fixture repository {}", root.display()))
}

#[path = "../../../../tests/support/mod.rs"]
mod runtime_repository;

#[test]
fn assignment_fixture_root_matches_declared_runtime_checkout() -> Result<()> {
    let declared = runtime_repository::repo_root().canonicalize()?;
    let observed = repo_root()?.canonicalize()?;
    assert_eq!(
        observed, declared,
        "BAM assignment fixture acquisition must use the declared runtime checkout"
    );
    Ok(())
}
