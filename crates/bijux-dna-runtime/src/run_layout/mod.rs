#![allow(dead_code)]

mod contracts;
mod journal;

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::RunMetadataV1;

use crate::recording::write_canonical_json;
pub use contracts::*;
pub use journal::{append_event, update_run_index};

/// Create the canonical run layout under the base directory.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn create_run_layout(base_dir: &Path) -> Result<(String, RunLayout)> {
    let run_id = Uuid::new_v4().to_string();
    let run_dir = bijux_dna_infra::run_layout_paths(base_dir, &run_id).run_dir;
    let stages_dir = run_dir.join("stages");
    let summary_dir = run_dir.join("summary");
    bijux_dna_infra::ensure_dir(&stages_dir).context("create run stages dir")?;
    bijux_dna_infra::ensure_dir(&summary_dir).context("create run summary dir")?;
    let layout = RunLayout {
        assessment_path: run_dir.join("input_assessment.json"),
        manifest_path: run_dir.join("execution_manifest.json"),
        environment_path: run_dir.join("environment.json"),
        metadata_path: run_dir.join("run_metadata.json"),
        events_path: run_dir.join("events.jsonl"),
        stages_dir,
        summary_dir,
        run_dir,
    };
    Ok((run_id, layout))
}

/// Write the environment fingerprint.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_environment(layout: &RunLayout, env: &RunEnvironment) -> Result<()> {
    write_canonical_json(&layout.environment_path, env)?;
    Ok(())
}

/// Write run metadata.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_run_metadata(layout: &RunLayout, metadata: &RunMetadataV1) -> Result<()> {
    write_canonical_json(&layout.metadata_path, metadata)?;
    Ok(())
}

/// Write the run manifest.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_manifest(layout: &RunLayout, manifest: &RunManifest) -> Result<()> {
    let payload = to_canonical_json_bytes(manifest)?;
    bijux_dna_infra::atomic_write_bytes(&layout.manifest_path, payload.as_slice())?;
    Ok(())
}

#[must_use]
pub fn now_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}
