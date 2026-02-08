use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sha2::Digest;

use bijux_dna_infra::bench_tools_dir;

use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::hashing::input_fingerprint;
use bijux_dna_core::prelude::CacheKey;

use super::io::{hash_file_sha256, write_canonical_json};

#[derive(Debug)]
pub struct RunDirs {
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub metrics_path: PathBuf,
    pub run_manifest_path: PathBuf,
}

#[derive(Debug)]
pub struct RunArtifactInput {
    pub name: &'static str,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct PlanArtifacts {
    pub plan_path: PathBuf,
    pub effective_config_path: PathBuf,
    pub stage_config_path: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct ObservabilityManifestV1 {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub tool_id: String,
    pub artifacts: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ProgressEventV1 {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub tool_id: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub outputs: Vec<String>,
    pub metrics_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunsExportRowV1 {
    pub schema_version: &'static str,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub started_at: String,
    pub finished_at: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub params_hash: String,
    pub input_hash: String,
    pub metrics_path: Option<String>,
}

#[must_use]
pub fn compute_run_id(
    stage: &str,
    tool: &str,
    image_digest: &str,
    input_hash: &str,
    params_hash: &str,
) -> String {
    let seed = format!("{stage}|{tool}|{image_digest}|{input_hash}|{params_hash}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// # Errors
/// Returns an error if run directories cannot be created.
pub fn prepare_tool_run_dirs(tools_root: &Path, tool: &str, run_id: &str) -> Result<RunDirs> {
    let tool_dir = tools_root.join(tool);
    let run_dir = tool_dir.join("run").join(run_id);
    let artifacts_dir = run_dir.join("artifacts");
    let logs_dir = run_dir.join("logs");
    bijux_dna_infra::ensure_dir(&artifacts_dir).context("create artifacts dir")?;
    bijux_dna_infra::ensure_dir(&logs_dir).context("create logs dir")?;
    Ok(RunDirs {
        artifacts_dir,
        logs_dir: logs_dir.clone(),
        manifest_path: run_dir.join("manifest.json"),
        metrics_path: run_dir.join("metrics.json"),
        run_manifest_path: run_dir.join("run_manifest.json"),
    })
}

/// # Errors
/// Returns an error if the run manifest or auxiliary files cannot be written.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
pub fn write_run_manifest(
    run_dirs: &RunDirs,
    stage: &str,
    _tool: &str,
    run_provenance: &crate::RunProvenanceV1,
    stage_contract_hash: Option<String>,
    extra_artifacts: &[RunArtifactInput],
) -> Result<()> {
    let mut artifacts = Vec::new();
    let manifest_hash = hash_file_sha256(&run_dirs.manifest_path)?;
    artifacts.push(serde_json::json!({
        "name": "execution_manifest",
        "path": run_dirs.manifest_path,
        "sha256": manifest_hash
    }));
    let metrics_hash = hash_file_sha256(&run_dirs.metrics_path)?;
    artifacts.push(serde_json::json!({
        "name": "metrics",
        "path": run_dirs.metrics_path,
        "sha256": metrics_hash
    }));
    for artifact in extra_artifacts {
        let hash = hash_file_sha256(&artifact.path)?;
        artifacts.push(serde_json::json!({
            "name": artifact.name,
            "path": artifact.path,
            "sha256": hash
        }));
    }
    let pipeline_id = std::env::var("BIJUX_PIPELINE_ID")
        .ok()
        .unwrap_or_else(|| run_provenance.pipeline_id.clone());
    let profile_id = std::env::var("BIJUX_PROFILE_ID").unwrap_or_else(|_| "unknown".to_string());
    let graph_hash = run_provenance
        .plan_hash
        .clone()
        .or_else(|| std::env::var("BIJUX_PLAN_HASH").ok())
        .unwrap_or_else(|| "unknown".to_string());
    let cache_key = CacheKey::new(
        input_fingerprint(&run_provenance.input_hashes),
        run_provenance.params_hash.clone(),
        run_provenance.tool_version.clone(),
        run_provenance
            .tool_image_digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
    );
    let tool_invocations = {
        let path = run_dirs.artifacts_dir.join("tool_invocation.json");
        if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            let parsed: ToolInvocationV1 = serde_json::from_str(&raw)?;
            vec![parsed]
        } else {
            Vec::new()
        }
    };
    let output_artifacts: Vec<serde_json::Value> = artifacts
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "stage_id": stage,
                "name": artifact.get("name").cloned().unwrap_or_default(),
                "role": "unknown",
                "optional": false,
                "path": artifact.get("path").cloned().unwrap_or_default(),
                "sha256": artifact.get("sha256").cloned().unwrap_or_default(),
            })
        })
        .collect();
    let payload = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": "unknown",
        "pipeline_id": pipeline_id,
        "profile_id": profile_id,
        "graph_hash": graph_hash,
        "cache_key": cache_key,
        "stage_contract_hash": stage_contract_hash,
        "toolchain_versions": {
            "planner": std::env::var("BIJUX_PLANNER_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            "engine": std::env::var("BIJUX_ENGINE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        },
        "dataset_fingerprints": run_provenance.input_hashes.clone(),
        "tool_invocations": tool_invocations,
        "output_artifacts": output_artifacts,
        "stages": [],
        "failures": [],
        "run_provenance": run_provenance,
        "telemetry": {
            "events_jsonl": run_artifacts_dir(run_dirs)?.join("telemetry").join("events.jsonl"),
        },
        "dashboard": {
            "facts_jsonl": run_artifacts_dir(run_dirs)?.join("dashboard").join("facts.jsonl"),
        },
    });
    let telemetry_dir = run_artifacts_dir(run_dirs)?.join("telemetry");
    bijux_dna_infra::ensure_dir(&telemetry_dir).context("create telemetry dir")?;
    write_canonical_json(&telemetry_dir.join("timings.json"), &serde_json::json!([]))
        .context("write timings.json")?;
    write_canonical_json(
        &telemetry_dir.join("resources.json"),
        &serde_json::json!([]),
    )
    .context("write resources.json")?;
    write_canonical_json(&telemetry_dir.join("errors.json"), &serde_json::json!([]))
        .context("write errors.json")?;
    super::io::write_atomic_bytes(&telemetry_dir.join("events.jsonl"), b"")
        .context("write events.jsonl")?;
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    super::io::write_atomic_bytes(&run_dirs.run_manifest_path, payload.as_slice())
        .context("write run_manifest.json")?;
    Ok(())
}

/// # Errors
/// Returns an error if JSON serialization or writing fails.
pub fn write_stage_plan_json<T: Serialize>(
    run_dirs: &RunDirs,
    file_name: &str,
    plan: &T,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dirs)?;
    let plans_dir = root.join("plans");
    bijux_dna_infra::ensure_dir(&plans_dir).context("create plans artifact dir")?;
    let path = plans_dir.join(file_name);
    bijux_dna_infra::ensure_dir(path.parent().unwrap_or(&plans_dir))
        .context("create plan parent dir")?;
    write_canonical_json(&path, plan).context("write stage plan json")?;
    Ok(path)
}

#[must_use]
pub fn run_artifacts_dir_for_out(out_dir: &Path) -> PathBuf {
    out_dir.join("run_artifacts")
}

#[allow(dead_code)]
#[must_use]
pub fn tool_run_artifacts_dir(
    out: &Path,
    stage: &str,
    sample_id: &str,
    tool: &str,
    run_id: &str,
) -> PathBuf {
    bench_tools_dir(out, stage, sample_id)
        .join(tool)
        .join("run")
        .join(run_id)
        .join("artifacts")
}

fn run_artifacts_dir(run_dirs: &RunDirs) -> Result<PathBuf> {
    let run_dir = run_dirs
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run dir missing for manifest"))?;
    Ok(run_dir.join("run_artifacts"))
}
