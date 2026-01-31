//! Owner: bijux-bench
//! Deterministic, atomic artifact writers.
//! Owns bench output serialization.
//! Must not perform analysis logic.
//! Invariants: writes are atomic and stable.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::contract::BenchmarkSummary;
use crate::model::BenchObservation;

fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("missing parent for {}", path.display()))?;
    fs::create_dir_all(dir)?;
    let mut temp = PathBuf::from(path);
    temp.set_extension("tmp");
    let mut file = File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temp, path)?;
    Ok(())
}

/// Write observations as deterministic JSONL.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_observations_jsonl(path: &Path, observations: &[BenchObservation]) -> Result<()> {
    let mut ordered = observations.to_vec();
    ordered.sort_by(|a, b| {
        (
            &a.run_id,
            &a.tool,
            &a.params_hash,
            &a.dataset_hash,
            a.replicate_index,
        )
            .cmp(&(
                &b.run_id,
                &b.tool,
                &b.params_hash,
                &b.dataset_hash,
                b.replicate_index,
            ))
    });
    let mut payload = String::new();
    for obs in ordered {
        payload.push_str(&serde_json::to_string(&obs)?);
        payload.push('\n');
    }
    write_atomic_bytes(path, payload.as_bytes())
        .with_context(|| format!("write observations {}", path.display()))
}

/// Write summary JSON.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_summary_json(path: &Path, summary: &BenchmarkSummary) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(summary)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write summary {}", path.display()))
}
