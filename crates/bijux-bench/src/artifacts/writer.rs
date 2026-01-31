//! Owner: bijux-bench
//! Deterministic, atomic artifact writers.
//! Owns bench output serialization.
//! Must not perform analysis logic.
//! Invariants: writes are atomic and stable.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use std::collections::BTreeSet;

use crate::model::{BenchmarkDecision, BenchmarkObservation, BenchmarkSummary};

type ObservationKey = (String, String, String, String, String);

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
#[derive(Debug, Clone, Copy)]
pub enum WriteMode {
    Resume,
    Force,
}

fn observation_key(obs: &BenchmarkObservation) -> ObservationKey {
    (
        obs.dataset_id.clone(),
        obs.stage_id.clone(),
        obs.tool_id.clone(),
        obs.params_hash.clone(),
        obs.replicate_id.clone(),
    )
}

fn canonical_json_line<T: serde::Serialize>(value: &T) -> Result<String> {
    let json = serde_json::to_value(value)?;
    let canonical = bijux_core::canonicalize_json_value(&json);
    Ok(serde_json::to_string(&canonical)?)
}

fn load_existing_keys(path: &Path) -> Result<BTreeSet<ObservationKey>> {
    let mut keys = BTreeSet::new();
    if !path.exists() {
        return Ok(keys);
    }
    let raw = std::fs::read_to_string(path)?;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)?;
        let key = (
            value
                .get("dataset_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("stage_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("tool_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("params_hash")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            value
                .get("replicate_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        );
        keys.insert(key);
    }
    Ok(keys)
}

/// Read observations from JSONL.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn read_observations_jsonl(path: &Path) -> Result<Vec<BenchmarkObservation>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    let mut observations = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let obs: BenchmarkObservation = serde_json::from_str(line)?;
        observations.push(obs);
    }
    Ok(observations)
}

pub fn write_observations_jsonl(
    path: &Path,
    observations: &[BenchmarkObservation],
    mode: WriteMode,
) -> Result<()> {
    let mut ordered = observations.to_vec();
    ordered.sort_by(|a, b| {
        (
            &a.dataset_id,
            &a.stage_id,
            &a.tool_id,
            &a.params_hash,
            &a.replicate_id,
            a.replicate_index,
        )
            .cmp(&(
                &b.dataset_id,
                &b.stage_id,
                &b.tool_id,
                &b.params_hash,
                &b.replicate_id,
                b.replicate_index,
            ))
    });
    let existing = if matches!(mode, WriteMode::Resume) {
        load_existing_keys(path)?
    } else {
        BTreeSet::new()
    };
    let mut payload = String::new();
    for obs in ordered {
        if matches!(mode, WriteMode::Resume) && existing.contains(&observation_key(&obs)) {
            continue;
        }
        payload.push_str(&canonical_json_line(&obs)?);
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
    let json = serde_json::to_value(summary)?;
    let canonical = bijux_core::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write summary {}", path.display()))
}

pub fn write_decision_json(path: &Path, decision: &BenchmarkDecision) -> Result<()> {
    let json = serde_json::to_value(decision)?;
    let canonical = bijux_core::canonicalize_json_value(&json);
    let bytes = serde_json::to_vec_pretty(&canonical)?;
    write_atomic_bytes(path, &bytes).with_context(|| format!("write decision {}", path.display()))
}
