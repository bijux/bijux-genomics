mod manifest_identity;
mod profile_lock;

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use bijux_dna_infra::bench_tools_dir;

use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::CacheKey;

use super::io::{hash_file_sha256, write_canonical_json};
pub use manifest_identity::compute_run_id;

use self::manifest_identity::{detect_run_context, input_hash_from_many, manifest_sort_key};
use self::profile_lock::write_profile_and_lock_manifests;

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
    let replay_tool_image_ref = tool_invocations.first().map(|inv| inv.tool_id.to_string());
    let replay_tool_image_digest = tool_invocations
        .first()
        .map(|inv| inv.image_digest.clone())
        .or_else(|| run_provenance.tool_image_digest.clone());
    let replay_tool_version_output = tool_invocations
        .first()
        .and_then(|inv| inv.resolved_tool_version.clone())
        .or_else(|| Some(run_provenance.tool_version.clone()));
    let replay_tool_id = tool_invocations.first().map(|inv| inv.tool_id.to_string());
    let image_upstream = std::env::var("BIJUX_IMAGE_UPSTREAM").ok();
    let image_build_timestamp_unix_s = std::env::var("BIJUX_IMAGE_BUILD_TIMESTAMP_UNIX_S")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());
    let reproducibility_dir = run_artifacts_dir(run_dirs)?.join("reproducibility");
    bijux_dna_infra::ensure_dir(&reproducibility_dir).context("create reproducibility dir")?;
    let reproducibility_report_path = reproducibility_dir.join("report.json");
    let profile_hash = std::env::var("BIJUX_PROFILE_HASH").ok();
    let reproducibility_tuple = serde_json::json!({
        "schema_version": "bijux.repro_tuple.v1",
        "tool_digests": tool_invocations
            .iter()
            .map(|inv| serde_json::json!({
                "stage_id": inv.stage_id,
                "tool_id": inv.tool_id,
                "image_digest": inv.image_digest,
            }))
            .collect::<Vec<_>>(),
        "bank_hashes": serde_json::json!({}),
        "profile_hash": profile_hash,
    });
    let run_context = detect_run_context()?;
    if matches!(run_context, crate::RunContextV1::Hpc { .. }) {
        let has_tool_digests = reproducibility_tuple
            .get("tool_digests")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|v| !v.is_empty());
        let has_profile_hash = reproducibility_tuple
            .get("profile_hash")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|v| !v.trim().is_empty());
        if !has_tool_digests || !has_profile_hash {
            return Err(anyhow!(
                "missing reproducibility tuple for HPC run (tool digests + profile hash required)"
            ));
        }
    }
    let reproducibility_identity = bijux_dna_core::prelude::ReproducibilityIdentityV1 {
        image_digest: replay_tool_image_digest
            .clone()
            .unwrap_or_else(|| declared_tool_image_digest.clone()),
        tool_version: run_provenance.tool_version.clone(),
        params_hash: run_provenance.params_hash.clone(),
        input_hash: input_hash_from_many(&run_provenance.input_hashes),
        bank_hashes: serde_json::json!({}),
    };
    write_canonical_json(
        &reproducibility_report_path,
        &serde_json::json!({
            "schema_version": "bijux.reproducibility_report.v1",
            "pipeline_id": pipeline_id,
            "plan_hash": graph_hash,
            "params_hash": run_provenance.params_hash,
            "input_hashes": run_provenance.input_hashes,
            "tool_version": run_provenance.tool_version,
            "tool_image_digest": run_provenance.tool_image_digest,
            "tool_invocations": tool_invocations.clone(),
            "reproducibility_identity": reproducibility_identity,
            "reproducibility_tuple": reproducibility_tuple.clone(),
        }),
    )
    .context("write reproducibility report")?;
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
        "tool_invocations": tool_invocations,
        "output_artifacts": [],
        "stages": [],
        "failures": [],
        "run_provenance": run_provenance,
        "execution_replay_identity": {
            "tool_image_ref": replay_tool_image_ref,
            "tool_image_digest": replay_tool_image_digest,
            "tool_version_output": replay_tool_version_output,
        },
        "image_identity_provenance": {
            "tool_id": replay_tool_id,
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
        "run_context": run_context,
        "reproducibility_tuple": reproducibility_tuple,
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

fn collect_all_run_artifacts(
    run_dirs: &RunDirs,
    extra_artifacts: &[RunArtifactInput],
) -> Result<Vec<serde_json::Value>> {
    let mut out = Vec::new();
    out.push(make_artifact_record(
        "execution_manifest",
        &run_dirs.manifest_path,
    )?);
    out.push(make_artifact_record("metrics", &run_dirs.metrics_path)?);
    for artifact in extra_artifacts {
        out.push(make_artifact_record(artifact.name, &artifact.path)?);
    }
    let run_artifacts = run_artifacts_dir(run_dirs)?;
    if run_artifacts.exists() {
        for path in collect_files_sorted(&run_artifacts)? {
            let rel = path
                .strip_prefix(&run_artifacts)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or("artifact");
            let name = format!("run_artifacts/{rel}");
            out.push(make_artifact_record(&name, &path)?);
        }
    }
    out.sort_by_key(|artifact| manifest_sort_key(artifact, "name"));
    Ok(out)
}

fn make_artifact_record(name: &str, path: &Path) -> Result<serde_json::Value> {
    let hash = hash_file_sha256(path)?;
    Ok(serde_json::json!({
        "name": name,
        "path": path,
        "sha256": hash
    }))
}

fn collect_files_sorted(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).with_context(|| format!("read dir {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files_sorted(&path)?);
        } else if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
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
