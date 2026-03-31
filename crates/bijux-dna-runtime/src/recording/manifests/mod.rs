mod artifact_catalog;
mod manifest_identity;
mod profile_lock;
mod reproducibility;

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::CacheKey;
use serde::Serialize;

pub use self::artifact_catalog::{
    run_artifacts_dir_for_out, tool_run_artifacts_dir, write_stage_plan_json,
};
use super::io::write_canonical_json;
pub use manifest_identity::compute_run_id;

use self::artifact_catalog::{collect_all_run_artifacts, run_artifacts_dir};
use self::manifest_identity::input_hash_from_many;
use self::profile_lock::write_profile_and_lock_manifests;
use self::reproducibility::{prepare_reproducibility_context, write_reproducibility_report};

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
    let run_id = run_dirs
        .run_manifest_path
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("run id missing from run manifest path"))?
        .to_string();
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
    let dashboard_dir = run_artifacts_dir(run_dirs)?.join("dashboard");
    bijux_dna_infra::ensure_dir(&dashboard_dir).context("create dashboard dir")?;
    super::io::write_atomic_bytes(&dashboard_dir.join("facts.jsonl"), b"")
        .context("write facts.jsonl")?;
    let pipeline_id = std::env::var("BIJUX_PIPELINE_ID")
        .ok()
        .unwrap_or_else(|| run_provenance.pipeline_id.clone());
    let profile_id = std::env::var("BIJUX_PROFILE_ID").ok();
    let graph_hash = run_provenance
        .plan_hash
        .clone()
        .or_else(|| std::env::var("BIJUX_PLAN_HASH").ok());
    let declared_tool_image_digest = run_provenance
        .tool_image_digest
        .clone()
        .ok_or_else(|| anyhow!("run manifest requires declared tool image digest"))?;
    let cache_key = CacheKey::new(
        input_hash_from_many(&run_provenance.input_hashes),
        run_provenance.params_hash.clone(),
        run_provenance.tool_version.clone(),
        declared_tool_image_digest.clone(),
    );
    let repro_context = prepare_reproducibility_context(run_dirs, run_provenance)?;
    let image_upstream = std::env::var("BIJUX_IMAGE_UPSTREAM").ok();
    let image_build_timestamp_unix_s = std::env::var("BIJUX_IMAGE_BUILD_TIMESTAMP_UNIX_S")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());
    write_reproducibility_report(
        run_dirs,
        &pipeline_id,
        graph_hash.as_ref(),
        run_provenance,
        &declared_tool_image_digest,
        &repro_context.tool_invocations,
        repro_context.replay_tool_image_digest.as_ref(),
        &repro_context.reproducibility_tuple,
    )?;
    let payload = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": run_id,
        "pipeline_id": pipeline_id,
        "profile_id": profile_id,
        "graph_hash": graph_hash,
        "cache_key": cache_key,
        "stage_contract_hash": stage_contract_hash,
        "toolchain_versions": {
            "planner": std::env::var("BIJUX_PLANNER_VERSION").ok(),
            "engine": std::env::var("BIJUX_ENGINE_VERSION").ok(),
        },
        "dataset_fingerprints": run_provenance.input_hashes.clone(),
        "tool_invocations": repro_context.tool_invocations,
        "output_artifacts": [],
        "stages": [],
        "failures": [],
        "run_provenance": run_provenance,
        "execution_replay_identity": {
            "tool_image_ref": repro_context.replay_tool_image_ref,
            "tool_image_digest": repro_context.replay_tool_image_digest,
            "tool_version_output": repro_context.replay_tool_version_output,
        },
        "image_identity_provenance": {
            "tool_id": repro_context.replay_tool_id,
            "version": run_provenance.tool_version.clone(),
            "digest": run_provenance.tool_image_digest.clone(),
            "upstream": image_upstream,
            "build_timestamp_unix_s": image_build_timestamp_unix_s,
        },
        "telemetry": {
            "events_jsonl": run_artifacts_dir(run_dirs)?.join("telemetry").join("events.jsonl"),
        },
        "dashboard": {
            "facts_jsonl": run_artifacts_dir(run_dirs)?.join("dashboard").join("facts.jsonl"),
        },
        "run_context": repro_context.run_context,
        "reproducibility_tuple": repro_context.reproducibility_tuple,
    });
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    super::io::write_atomic_bytes(&run_dirs.run_manifest_path, payload.as_slice())
        .context("write run_manifest.json")?;
    let artifacts = collect_all_run_artifacts(run_dirs, extra_artifacts)?;
    let output_artifacts: Vec<serde_json::Value> = artifacts
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "stage_id": stage,
                "name": artifact.get("name").cloned().unwrap_or(serde_json::Value::Null),
                "role": serde_json::Value::Null,
                "optional": false,
                "path": artifact.get("path").cloned().unwrap_or(serde_json::Value::Null),
                "sha256": artifact.get("sha256").cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .collect();
    let mut run_manifest: serde_json::Value =
        serde_json::from_slice(std::fs::read(&run_dirs.run_manifest_path)?.as_slice())
            .context("parse run_manifest for artifact enrichment")?;
    run_manifest["output_artifacts"] = serde_json::Value::Array(output_artifacts);
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&run_manifest)?;
    super::io::write_atomic_bytes(&run_dirs.run_manifest_path, payload.as_slice())
        .context("rewrite run_manifest with complete artifact hashes")?;
    write_profile_and_lock_manifests(&run_dirs.run_manifest_path)?;
    Ok(())
}
