#![cfg(test)]
use anyhow::{anyhow, Result};
#[test]
fn foreign_root_is_rejected_before_retained_evidence_effects() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let error = super::context(dir.path()).err().ok_or_else(|| anyhow!("foreign root admitted"))?;
    assert!(error.to_string().contains("differs from the requested evidence root"));
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
    Ok(())
}
