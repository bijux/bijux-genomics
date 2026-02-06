#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use uuid::Uuid;

use bijux_core::contract::ContractVersion;
use bijux_core::metadata::RunMetadataV1;
use bijux_core::metrics::ToolInvocationV1;
use bijux_core::primitives::input_assessment::FastqLayout;
use bijux_core::primitives::{to_canonical_json_bytes, CacheKey, Result as CoreResult};

use crate::events::RunEvent;

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
    pub execution_metrics_path: PathBuf,
    pub domain_metrics_path: PathBuf,
    pub logs_dir: PathBuf,
    pub outputs_dir: PathBuf,
    pub tool_invocation_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: String,
    pub contract_version: ContractVersion,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub pipeline: String,
    pub graph_hash: String,
    #[serde(default)]
    pub cache_key: Option<CacheKey>,
    pub layout: FastqLayout,
    pub stages: Vec<RunStageEntry>,
    #[serde(default)]
    pub tool_invocations: Vec<ToolInvocationV1>,
    #[serde(default)]
    pub artifacts: Vec<RunArtifactEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifactEntry {
    pub name: String,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexEntry {
    pub run_id: String,
    pub domain: String,
    pub pipeline: String,
    pub stages: Vec<String>,
    pub layout: FastqLayout,
    pub tools: Vec<String>,
    pub objective: Option<String>,
    pub platform: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexLine {
    pub schema_version: u32,
    pub run: RunIndexEntry,
}

#[derive(Debug, Clone)]
pub struct RunLayout {
    pub run_dir: PathBuf,
    pub stages_dir: PathBuf,
    pub summary_dir: PathBuf,
    pub assessment_path: PathBuf,
    pub manifest_path: PathBuf,
    pub environment_path: PathBuf,
    pub metadata_path: PathBuf,
    pub events_path: PathBuf,
}

/// Create the canonical run layout under the base directory.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn create_run_layout(base_dir: &Path) -> Result<(String, RunLayout)> {
    let run_id = Uuid::new_v4().to_string();
    let run_dir = bijux_infra::run_layout_paths(base_dir, &run_id).run_dir;
    let stages_dir = run_dir.join("stages");
    let summary_dir = run_dir.join("summary");
    bijux_infra::ensure_dir(&stages_dir).context("create run stages dir")?;
    bijux_infra::ensure_dir(&summary_dir).context("create run summary dir")?;
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
    let payload = serde_json::to_string_pretty(env)?;
    bijux_infra::atomic_write_bytes(&layout.environment_path, payload.as_bytes())?;
    Ok(())
}

/// Write run metadata.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_run_metadata(layout: &RunLayout, metadata: &RunMetadataV1) -> Result<()> {
    let payload = serde_json::to_string_pretty(metadata)?;
    bijux_infra::atomic_write_bytes(&layout.metadata_path, payload.as_bytes())?;
    Ok(())
}

/// Write the run manifest.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_manifest(layout: &RunLayout, manifest: &RunManifest) -> Result<()> {
    let payload = to_canonical_json_bytes(manifest)?;
    bijux_infra::atomic_write_bytes(&layout.manifest_path, payload.as_slice())?;
    Ok(())
}

impl RunManifest {
    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate(&self) -> CoreResult<()> {
        if self.graph_hash.trim().is_empty() {
            return Err(bijux_core::primitives::BijuxError::validation(
                "run manifest graph_hash is empty",
            ));
        }
        if self.artifacts.is_empty() {
            return Err(bijux_core::primitives::BijuxError::validation(
                "run manifest artifacts list is empty",
            ));
        }
        for artifact in &self.artifacts {
            if artifact.name.trim().is_empty() {
                return Err(bijux_core::primitives::BijuxError::validation(
                    "run manifest artifact name is empty",
                ));
            }
            if artifact.sha256.trim().is_empty() {
                return Err(bijux_core::primitives::BijuxError::validation(
                    "run manifest artifact hash is empty",
                ));
            }
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if canonical serialization fails.
    pub fn hash(&self) -> CoreResult<String> {
        let bytes = to_canonical_json_bytes(self)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Append a run entry to `bijux-runs/index.jsonl`.
///
/// # Errors
/// Returns an error if the index cannot be written.
pub fn update_run_index(base_dir: &Path, entry: RunIndexEntry) -> Result<()> {
    let index_path = base_dir.join("bijux-runs").join("index.jsonl");
    if let Some(parent) = index_path.parent() {
        bijux_infra::ensure_dir(parent)?;
    }
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
    let payload = serde_json::to_string(event)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&layout.events_path)?;
    std::io::Write::write_all(&mut file, format!("{payload}\n").as_bytes())?;
    Ok(())
}

#[must_use]
pub fn now_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}
