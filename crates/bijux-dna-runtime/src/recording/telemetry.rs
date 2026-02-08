use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

/// # Errors
/// Returns an error if the telemetry event cannot be appended.
pub fn write_telemetry_event(path: &Path, event: &crate::TelemetryEventV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent).context("create telemetry dir")?;
    }
    let line = bijux_dna_core::contract::canonical::to_canonical_json_bytes(event)?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("open telemetry jsonl")?
        .write_all(format!("{}\n", String::from_utf8_lossy(&line)).as_bytes())
        .context("append telemetry jsonl")?;
    Ok(())
}
