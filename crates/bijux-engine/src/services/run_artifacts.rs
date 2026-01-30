use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::Digest;

use crate::services::composer::paths::bench_tools_dir;

use serde::Serialize;
use uuid::Uuid;

use bijux_core::{
    canonicalize_json_value, EffectiveConfigV1, FactsRowV1, RetentionReportV1,
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

pub fn params_hash(params: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(params).context("serialize params")?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
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
    std::fs::create_dir_all(&artifacts_dir).context("create artifacts dir")?;
    std::fs::create_dir_all(&logs_dir).context("create logs dir")?;
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
    )?;
    std::fs::copy(&path, &run_dirs.retention_report_path).context("copy retention_report.json")?;
    Ok(())
}

pub fn write_run_manifest(
    run_dirs: &RunDirs,
    stage: &str,
    tool: &str,
    adapter_bank_path: &Path,
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
        "artifacts": artifacts
    });
    std::fs::write(
        &run_dirs.run_manifest_path,
        serde_json::to_vec_pretty(&payload)?,
    )
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
    std::fs::create_dir_all(&plans_dir).context("create plans artifact dir")?;
    let path = plans_dir.join(file_name);
    std::fs::create_dir_all(path.parent().unwrap_or(&plans_dir))
        .context("create plan parent dir")?;
    std::fs::write(&path, serde_json::to_vec_pretty(plan)?).context("write stage plan json")?;
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
    let log_path = run_dirs.logs_dir.join("tool.log");
    if stderr.is_empty() {
        std::fs::write(&log_path, stdout).context("write tool.log")?;
    } else {
        std::fs::write(&log_path, format!("{stdout}\n--- stderr ---\n{stderr}"))
            .context("write tool.log")?;
    }
    Ok(())
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
    std::fs::write(&run_dirs.metrics_path, serde_json::to_vec_pretty(&payload)?)
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
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
) -> Result<PlanArtifacts> {
    std::fs::create_dir_all(run_artifacts_dir).context("create run_artifacts dir")?;
    let plan_path = run_artifacts_dir.join("plan.json");
    let effective_config_path = run_artifacts_dir.join("effective_config.json");
    let config_dir = run_artifacts_dir.join("config");
    std::fs::create_dir_all(&config_dir).context("create config artifact dir")?;
    let stage_config_path = config_dir.join(format!("{stage_id}.effective.json"));
    let payload = serde_json::json!({
        "stage_id": stage_id,
        "stage_version": stage_version,
        "tool_id": tool_id,
        "inputs": inputs,
        "outputs": outputs,
        "parameters": params,
    });
    std::fs::write(&plan_path, serde_json::to_vec_pretty(&payload)?).context("write plan.json")?;
    std::fs::write(&effective_config_path, serde_json::to_vec_pretty(params)?)
        .context("write effective_config.json")?;
    let effective_config = EffectiveConfigV1 {
        schema_version: "bijux.effective_config.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version,
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        parameters_json: params.clone(),
    };
    std::fs::write(
        &stage_config_path,
        serde_json::to_vec_pretty(&effective_config)?,
    )
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
    let canonical_params = canonicalize_json_value(&ctx.parameters_json);
    let payload = MetricsEnvelopeV1 {
        schema_version: "bijux.metrics_envelope.v1",
        stage_id: ctx.stage_id.clone(),
        stage_version: ctx.stage_version,
        tool_id: ctx.tool_id.clone(),
        tool_version: ctx.tool_version.clone(),
        input_hash: ctx.input_hash.clone(),
        params_hash: ctx.params_hash.clone(),
        parameters_json: canonical_params,
        execution: *execution,
        metrics: metrics.clone(),
        output_hashes: output_hashes.to_vec(),
    };
    let path = run_artifacts_dir.join("metrics_envelope.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write metrics_envelope.json")?;
    Ok(path)
}

pub fn write_stage_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    stage_version: i32,
    tool_id: &str,
    tool_version: &str,
    outputs: &[PathBuf],
) -> Result<PathBuf> {
    let payload = StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version,
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
        outputs: outputs.iter().map(|p| p.display().to_string()).collect(),
        subreports: Vec::new(),
    };
    let path = run_artifacts_dir.join("stage_report.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write stage_report.json")?;
    Ok(path)
}

pub fn write_retention_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    tool_version: &str,
    params: &serde_json::Value,
) -> Result<PathBuf> {
    let payload = RetentionReportV1 {
        schema_version: "bijux.retention_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        boundary: "pre/post".to_string(),
        numerator: serde_json::json!({ "reads_out": null }),
        denominator: serde_json::json!({ "reads_in": null }),
        scope: "reads".to_string(),
        params: params.clone(),
    };
    let path = run_artifacts_dir.join("retention_report.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write retention_report.json")?;
    Ok(path)
}

pub fn write_telemetry_event(path: &Path, event: &TelemetryEventV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create telemetry dir")?;
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

pub fn write_facts_jsonl(path: &Path, fact: &FactsRowV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create dashboard dir")?;
    }
    let line = serde_json::to_string(fact)?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("open facts jsonl")?
        .write_all(format!("{line}\n").as_bytes())
        .context("append facts jsonl")?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn write_observability_manifest(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    plan_path: &Path,
    effective_config_path: &Path,
    stage_config_path: &Path,
    metrics_envelope_path: &Path,
    stage_report_path: &Path,
    retention_report_path: Option<&Path>,
) -> Result<PathBuf> {
    let mut artifacts = vec![
        serde_json::json!({
            "name": "plan",
            "path": plan_path,
        }),
        serde_json::json!({
            "name": "effective_config",
            "path": effective_config_path,
        }),
        serde_json::json!({
            "name": "stage_config",
            "path": stage_config_path,
        }),
        serde_json::json!({
            "name": "metrics_envelope",
            "path": metrics_envelope_path,
        }),
        serde_json::json!({
            "name": "stage_report",
            "path": stage_report_path,
        }),
    ];
    if let Some(path) = retention_report_path {
        artifacts.push(serde_json::json!({
            "name": "retention_report",
            "path": path,
        }));
    }
    let payload = ObservabilityManifestV1 {
        schema_version: "bijux.observability_manifest.v1",
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        artifacts,
    };
    let path = run_artifacts_dir.join("observability_manifest.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write observability_manifest.json")?;
    Ok(path)
}

pub fn default_trace_ids() -> (String, String) {
    (Uuid::new_v4().to_string(), Uuid::new_v4().to_string())
}
