use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::contracts::FastqLayout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEnvironment {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub runner: String,
    pub platform: String,
    pub tool_images: Vec<ToolImageDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolImageDigest {
    pub tool: String,
    pub image: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStageEntry {
    pub stage_id: String,
    pub tool_id: String,
    pub metrics_path: PathBuf,
    pub logs_dir: PathBuf,
    pub outputs_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub run_id: String,
    pub timestamp: String,
    pub pipeline: String,
    pub layout: FastqLayout,
    pub stages: Vec<RunStageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexEntry {
    pub run_id: String,
    pub pipeline: String,
    pub stages: Vec<String>,
    pub layout: FastqLayout,
    pub tools: Vec<String>,
    pub objective: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndex {
    pub runs: Vec<RunIndexEntry>,
}

#[derive(Debug, Clone)]
pub struct RunLayout {
    pub run_dir: PathBuf,
    pub assessment_path: PathBuf,
    pub manifest_path: PathBuf,
    pub environment_path: PathBuf,
    pub summary_dir: PathBuf,
    pub stages_dir: PathBuf,
}

/// Create the canonical run layout under the base directory.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn create_run_layout(base_dir: &Path) -> Result<(String, RunLayout)> {
    let timestamp = Utc::now();
    let run_id = format!(
        "run_{}_{}",
        timestamp.format("%Y%m%d%H%M%S"),
        Uuid::new_v4()
    );
    let run_dir = base_dir.join("runs").join(&run_id);
    let summary_dir = run_dir.join("summary");
    let stages_dir = run_dir.join("stages");
    std::fs::create_dir_all(&summary_dir).context("create run summary dir")?;
    std::fs::create_dir_all(&stages_dir).context("create run stages dir")?;
    let layout = RunLayout {
        assessment_path: run_dir.join("input_assessment.json"),
        manifest_path: run_dir.join("run_manifest.json"),
        environment_path: run_dir.join("environment.json"),
        summary_dir,
        stages_dir,
        run_dir,
    };
    Ok((run_id, layout))
}

/// Write the environment fingerprint.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_environment(layout: &RunLayout, env: &RunEnvironment) -> Result<()> {
    let payload = serde_json::to_string_pretty(env)?;
    std::fs::write(&layout.environment_path, payload)?;
    Ok(())
}

/// Write the run manifest.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_manifest(layout: &RunLayout, manifest: &RunManifest) -> Result<()> {
    let payload = serde_json::to_string_pretty(manifest)?;
    std::fs::write(&layout.manifest_path, payload)?;
    Ok(())
}

/// Append a run entry to `runs/index.json`.
///
/// # Errors
/// Returns an error if the index cannot be read or written.
pub fn update_run_index(base_dir: &Path, entry: RunIndexEntry) -> Result<()> {
    let index_path = base_dir.join("runs").join("index.json");
    let mut index = if index_path.exists() {
        let data = std::fs::read_to_string(&index_path)?;
        serde_json::from_str(&data).unwrap_or(RunIndex { runs: Vec::new() })
    } else {
        RunIndex { runs: Vec::new() }
    };
    index.runs.push(entry);
    let payload = serde_json::to_string_pretty(&index)?;
    std::fs::write(index_path, payload)?;
    Ok(())
}

#[must_use]
pub fn now_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}
