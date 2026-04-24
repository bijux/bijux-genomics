use anyhow::{Context, Result};

use super::artifact_catalog::run_artifacts_dir;
use super::runtime_support_files::RuntimeSupportFiles;
use super::RunDirs;
use crate::recording::{write_atomic_bytes, write_canonical_json};

/// # Errors
/// Returns an error if runtime support directories or files cannot be created.
pub(super) fn initialize_runtime_support_files(run_dirs: &RunDirs) -> Result<RuntimeSupportFiles> {
    let run_artifacts_dir = run_artifacts_dir(run_dirs)?;

    let telemetry_dir = run_artifacts_dir.join("telemetry");
    bijux_dna_infra::ensure_dir(&telemetry_dir).context("create telemetry dir")?;
    write_canonical_json(&telemetry_dir.join("timings.json"), &serde_json::json!([]))
        .context("write timings.json")?;
    write_canonical_json(&telemetry_dir.join("resources.json"), &serde_json::json!([]))
        .context("write resources.json")?;
    write_canonical_json(&telemetry_dir.join("errors.json"), &serde_json::json!([]))
        .context("write errors.json")?;
    let telemetry_events_path = telemetry_dir.join("events.jsonl");
    write_atomic_bytes(&telemetry_events_path, b"").context("write events.jsonl")?;

    let dashboard_dir = run_artifacts_dir.join("dashboard");
    bijux_dna_infra::ensure_dir(&dashboard_dir).context("create dashboard dir")?;
    let dashboard_facts_path = dashboard_dir.join("facts.jsonl");
    write_atomic_bytes(&dashboard_facts_path, b"").context("write facts.jsonl")?;

    Ok(RuntimeSupportFiles { telemetry_events_path, dashboard_facts_path })
}
