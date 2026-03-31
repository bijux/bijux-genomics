use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::run_layout::{RunIndexEntry, RunIndexLine, RunLayout};
use crate::telemetry::events::RunEvent;

/// Append a run entry to `bijux-dna-runs/index.jsonl`.
///
/// # Errors
/// Returns an error if the index cannot be written.
pub fn update_run_index(base_dir: &Path, entry: RunIndexEntry) -> Result<()> {
    let index_path = base_dir.join("bijux-dna-runs").join("index.jsonl");
    if let Some(parent) = index_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let _lock = bijux_dna_infra::FileLock::acquire(
        &index_path.with_extension("jsonl.lock"),
        Duration::from_secs(5),
    )
    .context("acquire run index lock")?;
    let line = RunIndexLine {
        schema_version: 1,
        run: entry,
    };
    let payload = serde_json::to_string(&line)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(index_path)?;
    std::io::Write::write_all(&mut file, format!("{payload}\n").as_bytes())?;
    Ok(())
}

/// Append an execution event to `events.jsonl`.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_event(layout: &RunLayout, event: &RunEvent) -> Result<()> {
    let _lock = bijux_dna_infra::FileLock::acquire(
        &layout.events_path.with_extension("jsonl.lock"),
        Duration::from_secs(5),
    )
    .context("acquire events jsonl lock")?;
    let payload = serde_json::to_string(event)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&layout.events_path)?;
    std::io::Write::write_all(&mut file, format!("{payload}\n").as_bytes())?;
    Ok(())
}
