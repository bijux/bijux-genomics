use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::Digest;

use crate::services::composer::paths::bench_tools_dir;

use serde::Serialize;
use uuid::Uuid;

use bijux_core::observability::{
    FilterReportV1, MergeReportV1, QcPostReportV1, TrimReportV1, ValidateReportV1,
};
use bijux_core::{
    metrics::AdapterBankProvenanceV1, EffectiveConfigV1, FactsRowV1, RetentionReportV1,
    StageObservabilityContextV1, StageReportV1, TelemetryEventV1,
};

#[derive(Debug)]
pub struct RunDirs {
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub metrics_path: PathBuf,
    pub run_manifest_path: PathBuf,
    pub retention_report_path: PathBuf,
}

#[derive(Debug)]
pub struct RunArtifactInput {
    pub name: &'static str,
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct MetricsEnvelopeV1 {
    pub schema_version: &'static str,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub context: bijux_core::metrics::MetricContextV1,
    pub input_hash: String,
    pub params_hash: String,
    pub parameters_json: serde_json::Value,
    pub execution: bijux_core::measure::ExecutionMetrics,
    pub metrics: serde_json::Value,
    pub output_hashes: Vec<String>,
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
        retention_report_path: run_dir.join("retention_report.json"),
    })
}

pub fn write_retention_report_placeholder(
    run_dirs: &RunDirs,
    stage: &str,
    tool: &str,
    params: &serde_json::Value,
) -> Result<()> {
    let path = write_retention_report_v1(
        &run_artifacts_dir(run_dirs)?,
        stage,
        tool,
        "unknown/TBD",
        params,
        params,
        0,
        0,
        0,
        0,
    )?;
    std::fs::copy(&path, &run_dirs.retention_report_path).context("copy retention_report.json")?;
    Ok(())
}

pub fn write_run_manifest(
    run_dirs: &RunDirs,
    stage: &str,
    tool: &str,
    adapter_bank_path: &Path,
    run_provenance: &bijux_core::RunProvenanceV1,
    extra_artifacts: &[RunArtifactInput],
) -> Result<()> {
    let mut artifacts = Vec::new();
    let has_retention_override = extra_artifacts
        .iter()
        .any(|artifact| artifact.name == "retention_report");
    let manifest_hash = crate::services::observer::hash_file_sha256(&run_dirs.manifest_path)?;
    artifacts.push(serde_json::json!({
        "name": "execution_manifest",
        "path": run_dirs.manifest_path,
        "sha256": manifest_hash
    }));
    let metrics_hash = crate::services::observer::hash_file_sha256(&run_dirs.metrics_path)?;
    artifacts.push(serde_json::json!({
        "name": "metrics",
        "path": run_dirs.metrics_path,
        "sha256": metrics_hash
    }));
    if !has_retention_override {
        let retention_hash =
            crate::services::observer::hash_file_sha256(&run_dirs.retention_report_path)?;
        artifacts.push(serde_json::json!({
            "name": "retention_report",
            "path": run_dirs.retention_report_path,
            "sha256": retention_hash
        }));
    }
    let adapter_hash = crate::services::observer::hash_file_sha256(adapter_bank_path)?;
    artifacts.push(serde_json::json!({
        "name": "adapter_bank",
        "path": adapter_bank_path,
        "sha256": adapter_hash
    }));
    for artifact in extra_artifacts {
        let hash = crate::services::observer::hash_file_sha256(&artifact.path)?;
        artifacts.push(serde_json::json!({
            "name": artifact.name,
            "path": artifact.path,
            "sha256": hash
        }));
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.run_manifest.v1",
        "stage": stage,
        "tool": tool,
        "artifacts": artifacts,
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
    bijux_infra::atomic_write_json(&telemetry_dir.join("resources.json"), &serde_json::json!([]))
        .context("write resources.json")?;
    bijux_infra::atomic_write_json(&telemetry_dir.join("errors.json"), &serde_json::json!([]))
        .context("write errors.json")?;
    bijux_infra::atomic_write_bytes(&telemetry_dir.join("events.jsonl"), b"")
        .context("write events.jsonl")?;
    bijux_infra::atomic_write_json(&run_dirs.run_manifest_path, &payload)
        .context("write run_manifest.json")?;
    Ok(())
}

fn run_artifacts_dir(run_dirs: &RunDirs) -> Result<PathBuf> {
    let run_dir = run_dirs
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run dir missing for manifest"))?;
    Ok(run_dir.join("run_artifacts"))
}
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

pub fn write_execution_logs(run_dirs: &RunDirs, stdout: &str, stderr: &str) -> Result<()> {
    let _ = write_execution_logs_bounded(&run_dirs.logs_dir, stdout, stderr)?;
    Ok(())
}

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
    bijux_infra::atomic_write_bytes(&combined_path, combined.as_bytes()).context("write tool.log")?;
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

pub fn write_metrics_json<T: serde::Serialize>(
    run_dirs: &RunDirs,
    execution: &bijux_core::measure::ExecutionMetrics,
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

pub fn run_artifacts_dir_for_out(out_dir: &Path) -> PathBuf {
    out_dir.join("run_artifacts")
}

#[allow(clippy::too_many_arguments)]
pub fn write_plan_artifacts(
    run_artifacts_dir: &Path,
    stage_id: &str,
    stage_version: i32,
    tool_id: &str,
    tool_version: &str,
    image_digest: Option<String>,
    runner: &str,
    platform: &str,
    resources: &bijux_core::ToolConstraints,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
    adapter_bank: Option<&bijux_core::metrics::AdapterBankProvenanceV1>,
    banks: Option<&serde_json::Value>,
    bank_assets: Option<&serde_json::Value>,
) -> Result<PlanArtifacts> {
    bijux_infra::ensure_dir(run_artifacts_dir).context("create run_artifacts dir")?;
    let plan_path = run_artifacts_dir.join("plan.json");
    let effective_config_path = run_artifacts_dir.join("effective_config.json");
    let config_dir = run_artifacts_dir.join("config");
    bijux_infra::ensure_dir(&config_dir).context("create config artifact dir")?;
    let stage_config_path = config_dir.join(format!("{stage_id}.effective.json"));
    let payload = serde_json::json!({
        "stage_id": stage_id,
        "stage_version": stage_version,
        "tool_id": tool_id,
        "inputs": inputs,
        "outputs": outputs,
        "parameters": params,
        "effective_params": effective_params,
    });
    bijux_infra::atomic_write_json(&plan_path, &payload).context("write plan.json")?;
    let effective_config = EffectiveConfigV1 {
        schema_version: "bijux.effective_config.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version,
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        image_digest,
        runner: runner.to_string(),
        platform: platform.to_string(),
        resources: resources.clone(),
        parameters_json: params.clone(),
        parameters_json_normalized: bijux_core::parameters_json_canonicalization(params),
        effective_params_json: effective_params.clone(),
        effective_params_json_normalized: bijux_core::parameters_json_canonicalization(
            effective_params,
        ),
        adapter_bank: adapter_bank.cloned(),
        banks: banks.cloned(),
        bank_assets: bank_assets.cloned(),
    };
    bijux_infra::atomic_write_json(&effective_config_path, &effective_config)
        .context("write effective_config.json")?;
    bijux_infra::atomic_write_json(&stage_config_path, &effective_config)
        .context("write effective config artifact")?;
    Ok(PlanArtifacts {
        plan_path,
        effective_config_path,
        stage_config_path,
    })
}

pub fn write_metrics_envelope(
    run_artifacts_dir: &Path,
    ctx: &StageObservabilityContextV1,
    execution: &bijux_core::measure::ExecutionMetrics,
    metrics: &serde_json::Value,
    output_hashes: &[String],
) -> Result<PathBuf> {
    let canonical_params = bijux_core::parameters_json_canonicalization(&ctx.parameters_json);
    let payload = MetricsEnvelopeV1 {
        schema_version: "bijux.metrics_envelope.v1",
        stage_id: ctx.stage_id.clone(),
        stage_version: ctx.stage_version,
        tool_id: ctx.tool_id.clone(),
        tool_version: ctx.tool_version.clone(),
        context: ctx.metric_context.clone(),
        input_hash: ctx.input_hash.clone(),
        params_hash: ctx.params_hash.clone(),
        parameters_json: canonical_params,
        execution: *execution,
        metrics: metrics.clone(),
        output_hashes: output_hashes.to_vec(),
    };
    let path = run_artifacts_dir.join("metrics_envelope.json");
    bijux_infra::atomic_write_json(&path, &payload).context("write metrics_envelope.json")?;
    Ok(path)
}

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

pub fn write_tool_invocation_json(
    run_artifacts_dir: &Path,
    stage_id: &str,
    invocation: &bijux_core::ToolInvocationV1,
) -> Result<PathBuf> {
    let invocations_dir = run_artifacts_dir.join("invocations");
    bijux_infra::ensure_dir(&invocations_dir).context("create invocations dir")?;
    let file_name = format!("{stage_id}.tool_invocation.json");
    let path = invocations_dir.join(file_name);
    bijux_infra::atomic_write_json(&path, invocation).context("write tool_invocation.json")?;
    Ok(path)
}
