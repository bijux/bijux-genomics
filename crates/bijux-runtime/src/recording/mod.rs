use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::Digest;

use bijux_infra::bench_tools_dir;

use crate::StageObservabilityContextV1;
use bijux_core::execution::execution_graph::ExecutionGraph;
use bijux_core::metrics::{MetricsEnvelope, ToolInvocationV1};
use bijux_core::primitives::hashing::params_hash;
use serde::Serialize;

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

/// Append a line to a JSONL file (create if missing).
///
/// # Errors
/// Returns an error if the file cannot be opened or written.
pub fn append_jsonl_line(path: &Path, line: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Write bytes atomically by writing a temp file and renaming.
///
/// # Errors
/// Returns an error if the target cannot be written.
pub fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("missing parent for {}", path.display()))?;
    bijux_infra::ensure_dir(dir)?;
    let mut temp = PathBuf::from(path);
    temp.set_extension("tmp");
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    bijux_infra::rename(&temp, path)?;
    Ok(())
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
    bijux_infra::ensure_dir(&artifacts_dir).context("create artifacts dir")?;
    bijux_infra::ensure_dir(&logs_dir).context("create logs dir")?;
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
pub fn write_run_manifest(
    run_dirs: &RunDirs,
    stage: &str,
    _tool: &str,
    run_provenance: &crate::RunProvenanceV1,
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
        "schema_version": "bijux.run_manifest.v2",
        "run_id": "unknown",
        "pipeline_id": pipeline_id,
        "profile_id": profile_id,
        "graph_hash": graph_hash,
        "toolchain_versions": {
            "planner": std::env::var("BIJUX_PLANNER_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            "engine": std::env::var("BIJUX_ENGINE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        },
        "dataset_fingerprints": run_provenance.input_hashes.clone(),
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
    bijux_infra::ensure_dir(&telemetry_dir).context("create telemetry dir")?;
    bijux_infra::atomic_write_json(&telemetry_dir.join("timings.json"), &serde_json::json!([]))
        .context("write timings.json")?;
    bijux_infra::atomic_write_json(
        &telemetry_dir.join("resources.json"),
        &serde_json::json!([]),
    )
    .context("write resources.json")?;
    bijux_infra::atomic_write_json(&telemetry_dir.join("errors.json"), &serde_json::json!([]))
        .context("write errors.json")?;
    bijux_infra::atomic_write_bytes(&telemetry_dir.join("events.jsonl"), b"")
        .context("write events.jsonl")?;
    bijux_infra::atomic_write_json(&run_dirs.run_manifest_path, &payload)
        .context("write run_manifest.json")?;
    Ok(())
}

/// # Errors
/// Returns an error if the provenance file cannot be written.
pub fn write_scientific_provenance(
    run_dir: &Path,
    provenance: &bijux_core::contract::ScientificProvenanceV1,
) -> Result<PathBuf> {
    let path = run_dir.join("scientific_provenance.json");
    bijux_infra::atomic_write_json(&path, provenance)
        .context("write scientific_provenance.json")?;
    Ok(path)
}

/// Build and write a minimal scientific provenance file derived from the plan.
///
/// This is intended for contract tests and dry-run validation.
/// # Errors
/// Returns an error if provenance serialization or writing fails.
pub fn write_plan_provenance(run_dir: &Path, plan: &ExecutionGraph) -> Result<PathBuf> {
    let mut invocations = Vec::new();
    let mut params_hashes = std::collections::BTreeMap::new();
    for step in plan.steps() {
        let params = serde_json::json!({ "command": step.command.template });
        let key = format!("{}:{}", step.step_id.0, step.image.image);
        let hash = params_hash(&params)?;
        params_hashes.insert(key, hash);
        let image_digest = step
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        invocations.push(ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            stage_id: step.step_id.to_string(),
            tool_id: "unknown".to_string(),
            tool_version: "unknown".to_string(),
            resolved_tool_version: None,
            image_digest,
            runner_kind: "fake".to_string(),
            platform: "unknown".to_string(),
            parameters_json: params.clone(),
            parameters_json_normalized: params.clone(),
            effective_params_json: params.clone(),
            effective_params_json_normalized: params.clone(),
            adapter_bank: None,
            banks: None,
            bank_assets: None,
            resources: step.resources.clone(),
            environment: std::collections::BTreeMap::new(),
            input_hashes: Vec::new(),
            output_hashes: Vec::new(),
            executed_command: None,
        });
    }
    let provenance = crate::provenance::build_scientific_provenance(
        plan.pipeline_id().to_string(),
        plan.planner_version().to_string(),
        &params_hashes,
        &invocations,
    );
    write_scientific_provenance(run_dir, &provenance)
}

/// # Errors
/// Returns an error if the telemetry event cannot be appended.
pub fn write_telemetry_event(path: &Path, event: &crate::TelemetryEventV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_infra::ensure_dir(parent).context("create telemetry dir")?;
    }
    let line = serde_json::to_string(event)?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("open telemetry jsonl")?
        .write_all(format!("{line}\n").as_bytes())
        .context("append telemetry jsonl")?;
    Ok(())
}

pub(crate) fn hash_file_sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let read = std::io::Read::read(&mut file, &mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn run_artifacts_dir(run_dirs: &RunDirs) -> Result<PathBuf> {
    let run_dir = run_dirs
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run dir missing for manifest"))?;
    Ok(run_dir.join("run_artifacts"))
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
    bijux_infra::ensure_dir(&plans_dir).context("create plans artifact dir")?;
    let path = plans_dir.join(file_name);
    bijux_infra::ensure_dir(path.parent().unwrap_or(&plans_dir))
        .context("create plan parent dir")?;
    bijux_infra::atomic_write_json(&path, plan).context("write stage plan json")?;
    Ok(path)
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

/// # Errors
/// Returns an error if execution logs cannot be written.
pub fn write_execution_logs(run_dirs: &RunDirs, stdout: &str, stderr: &str) -> Result<()> {
    let _ = write_execution_logs_bounded(&run_dirs.logs_dir, stdout, stderr)?;
    Ok(())
}

/// # Errors
/// Returns an error if bounded execution logs cannot be written.
pub fn write_execution_logs_bounded(
    logs_dir: &Path,
    stdout: &str,
    stderr: &str,
) -> Result<Vec<PathBuf>> {
    bijux_infra::ensure_dir(logs_dir).context("create logs dir")?;
    let tail_kb = log_tail_kb();
    let stdout_path = logs_dir.join("tool.stdout.log");
    let stderr_path = logs_dir.join("tool.stderr.log");
    let combined_path = logs_dir.join("tool.log");
    let stdout_tail = truncate_tail(stdout, tail_kb);
    let stderr_tail = truncate_tail(stderr, tail_kb);
    bijux_infra::atomic_write_bytes(&stdout_path, stdout_tail.as_bytes())
        .context("write tool.stdout.log")?;
    bijux_infra::atomic_write_bytes(&stderr_path, stderr_tail.as_bytes())
        .context("write tool.stderr.log")?;
    let combined = if stderr.is_empty() {
        truncate_tail(stdout, tail_kb)
    } else {
        truncate_tail(&format!("{stdout}\n--- stderr ---\n{stderr}"), tail_kb)
    };
    bijux_infra::atomic_write_bytes(&combined_path, combined.as_bytes())
        .context("write tool.log")?;
    Ok(vec![combined_path, stdout_path, stderr_path])
}

fn log_tail_kb() -> usize {
    std::env::var("BIJUX_LOG_TAIL_KB")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map_or(128, |value| value.clamp(1, 4096))
}

fn truncate_tail(text: &str, tail_kb: usize) -> String {
    let max_bytes = tail_kb.saturating_mul(1024);
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let bytes = text.as_bytes();
    let start = bytes.len().saturating_sub(max_bytes);
    String::from_utf8_lossy(&bytes[start..]).to_string()
}

/// # Errors
/// Returns an error if metrics JSON cannot be written.
pub fn write_metrics_json<T: serde::Serialize>(
    run_dirs: &RunDirs,
    execution: &bijux_core::primitives::measure::ExecutionMetrics,
    metrics: &bijux_core::metrics::MetricEnvelope<T>,
) -> Result<()> {
    let payload = serde_json::json!({
        "execution": execution,
        "metrics": metrics
    });
    bijux_infra::atomic_write_json(&run_dirs.metrics_path, &payload)
        .context("write metrics.json")?;
    Ok(())
}

#[must_use]
pub fn run_artifacts_dir_for_out(out_dir: &Path) -> PathBuf {
    out_dir.join("run_artifacts")
}

/// # Errors
/// Returns an error if the metrics envelope cannot be written.
pub fn write_metrics_envelope(
    run_artifacts_dir: &Path,
    ctx: &StageObservabilityContextV1,
    metrics: &serde_json::Value,
    input_hashes: &[String],
) -> Result<PathBuf> {
    let payload: MetricsEnvelope<serde_json::Value> = MetricsEnvelope {
        schema_version: "bijux.metrics_envelope.v2".to_string(),
        stage_id: ctx.stage_id.clone(),
        stage_version: ctx.stage_version,
        tool_id: ctx.tool_id.clone(),
        tool_version: ctx.tool_version.clone(),
        image_digest: ctx
            .metric_context
            .image_digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        parameters_fingerprint: ctx.parameters_fingerprint.clone(),
        input_fingerprint: ctx.input_fingerprint.clone(),
        parameters_json_normalized: ctx.parameters_json_normalized.clone(),
        input_hashes: input_hashes.to_vec(),
        metrics: metrics.clone(),
    };
    let path = run_artifacts_dir.join("metrics_envelope.json");
    bijux_infra::atomic_write_json(&path, &payload).context("write metrics_envelope.json")?;
    Ok(path)
}

/// # Errors
/// Returns an error if stage metrics cannot be written.
pub fn write_stage_metrics_json<T: serde::Serialize>(
    run_artifacts_dir: &Path,
    metrics: &bijux_core::metrics::StageMetricsV1<T>,
) -> Result<PathBuf> {
    let stage_path = run_artifacts_dir.join("stage_metrics.json");
    let metrics_path = run_artifacts_dir.join("metrics.json");
    let payload = serde_json::to_vec_pretty(metrics)?;
    bijux_infra::atomic_write_bytes(&stage_path, &payload).context("write stage_metrics.json")?;
    bijux_infra::atomic_write_bytes(&metrics_path, &payload).context("write metrics.json")?;
    Ok(stage_path)
}

/// # Errors
/// Returns an error if tool invocation JSON cannot be written.
pub fn write_tool_invocation_json(
    run_artifacts_dir: &Path,
    stage_id: &str,
    invocation: &ToolInvocationV1,
) -> Result<PathBuf> {
    let invocations_dir = run_artifacts_dir.join("invocations");
    bijux_infra::ensure_dir(&invocations_dir).context("create invocations dir")?;
    let file_name = format!("{stage_id}.tool_invocation.json");
    let path = invocations_dir.join(file_name);
    bijux_infra::atomic_write_json(&path, invocation).context("write tool_invocation.json")?;
    Ok(path)
}
