//! Owner: bijux-dna-bench
//! Immutable filesystem publication for derived benchmark stage reports.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

pub(in crate::workflow) const MAX_REPORT_INPUT_BYTES: usize = 16 * 1024 * 1024;

pub(in crate::workflow) fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    let metadata = fs::symlink_metadata(&canonical).with_context(|| format!("inspect {label}"))?;
    if !metadata.is_dir() {
        bail!("{label} is not a directory");
    }
    Ok(canonical)
}

pub(in crate::workflow) fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("create benchmark report directory {}", path.display()))?;
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark report directory {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("benchmark report output is not a regular directory");
    }
    Ok(())
}

pub(in crate::workflow) fn ensure_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark report publication is not a regular file");
            }
            let observed = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
            if observed != expected {
                bail!("benchmark report publication conflicts with immutable evidence");
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)
                .with_context(|| format!("publish benchmark report {}", path.display()))
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspect benchmark report {}", path.display()))
        }
    }
}
