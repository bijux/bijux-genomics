use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::RunMetadataV1;

use crate::recording::write_canonical_json;

use super::{RunEnvironment, RunManifest};

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
