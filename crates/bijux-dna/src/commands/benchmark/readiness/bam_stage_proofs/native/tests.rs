#![cfg(test)]

use super::{NativeAdapter, NativeProofKind, NativeProofWriter};
use anyhow::{Context, Result};

#[test]
fn mismatched_requested_root_rejects_before_native_proof_production() -> Result<()> {
    let requested = tempfile::tempdir()?;
    let error = NativeAdapter
        .write(requested.path(), NativeProofKind::Contamination)
        .err()
        .context("mismatched root must reject native proof production")?;
    assert_eq!(
        error.to_string(),
        "native BAM proof repository differs from the requested evidence root"
    );
    assert_eq!(std::fs::read_dir(requested.path())?.count(), 0);
    Ok(())
}
