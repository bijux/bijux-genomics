#![cfg(test)]
use super::super::admission::identity::{ensure_float_matches, ensure_paths_match};
use anyhow::{Context, Result};
use std::{fs, path::Path};

#[test]
fn threshold_equality_rejects_nonfinite_values_on_either_side() -> Result<()> {
    ensure_float_matches("stage", "threshold", 0.5, 0.5)?;
    for (observed, expected) in [
        (f64::NAN, 0.5),
        (0.5, f64::NAN),
        (f64::INFINITY, f64::INFINITY),
        (f64::NEG_INFINITY, 0.5),
        (0.6, 0.5),
    ] {
        let error = ensure_float_matches("stage", "threshold", observed, expected)
            .err()
            .context("invalid threshold must reject")?;
        assert!(error.to_string().contains("field `threshold`"));
    }
    Ok(())
}

#[test]
fn equal_file_contents_do_not_substitute_for_declared_path_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("declared.bam"), "same bytes")?;
    fs::write(root.path().join("other.bam"), "same bytes")?;
    ensure_paths_match(
        root.path(),
        "stage",
        "bam",
        Path::new("declared.bam"),
        &root.path().join("declared.bam"),
    )?;
    let error = ensure_paths_match(
        root.path(),
        "stage",
        "bam",
        Path::new("other.bam"),
        &root.path().join("declared.bam"),
    )
    .err()
    .context("different identity must reject")?;
    assert!(error.to_string().contains("must match"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn canonical_alias_preserves_the_declared_artifact_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("declared.bam"), "retained")?;
    std::os::unix::fs::symlink("declared.bam", root.path().join("alias.bam"))?;
    ensure_paths_match(
        root.path(),
        "stage",
        "bam",
        Path::new("alias.bam"),
        &root.path().join("declared.bam"),
    )?;
    Ok(())
}
