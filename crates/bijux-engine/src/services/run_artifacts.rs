use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sha2::Digest;

use crate::services::composer::paths::bench_tools_dir;

use serde::Serialize;
use uuid::Uuid;

use bijux_core::observability::{MergeReportV1, TrimReportV1, ValidateReportV1};
use bijux_core::{
    EffectiveConfigV1, FactsRowV1, RetentionReportV1, StageObservabilityContextV1, StageReportV1,
    TelemetryEventV1,
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

pub fn params_hash(params: &serde_json::Value) -> Result<String> {
    let canonical = bijux_core::parameters_json_canonicalization(params);
    let bytes = serde_json::to_vec(&canonical).context("serialize params")?;
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
    image_digest: Option<String>,
    runner: &str,
    platform: &str,
    resources: &bijux_core::ToolConstraints,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    adapter_bank: Option<&bijux_core::metrics::AdapterBankProvenanceV1>,
    banks: Option<&serde_json::Value>,
    bank_assets: Option<&serde_json::Value>,
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
        adapter_bank: adapter_bank.cloned(),
        banks: banks.cloned(),
        bank_assets: bank_assets.cloned(),
    };
    std::fs::write(
        &effective_config_path,
        serde_json::to_vec_pretty(&effective_config)?,
    )
    .context("write effective_config.json")?;
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
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write metrics_envelope.json")?;
    Ok(path)
}

pub fn write_stage_metrics_json<T: serde::Serialize>(
    run_artifacts_dir: &Path,
    metrics: &bijux_core::metrics::StageMetricsV1<T>,
) -> Result<PathBuf> {
    let stage_path = run_artifacts_dir.join("stage_metrics.json");
    let metrics_path = run_artifacts_dir.join("metrics.json");
    let payload = serde_json::to_vec_pretty(metrics)?;
    std::fs::write(&stage_path, &payload).context("write stage_metrics.json")?;
    std::fs::write(&metrics_path, &payload).context("write metrics.json")?;
    Ok(stage_path)
}

pub fn write_tool_invocation_json(
    run_artifacts_dir: &Path,
    stage_id: &str,
    invocation: &bijux_core::ToolInvocationV1,
) -> Result<PathBuf> {
    let invocations_dir = run_artifacts_dir.join("invocations");
    std::fs::create_dir_all(&invocations_dir).context("create invocations dir")?;
    let file_name = format!("{stage_id}.tool_invocation.json");
    let path = invocations_dir.join(file_name);
    std::fs::write(&path, serde_json::to_vec_pretty(invocation)?)
        .context("write tool_invocation.json")?;
    Ok(path)
}

pub fn write_stage_event_jsonl(
    run_artifacts_dir: &Path,
    event: &bijux_core::TelemetryEventV1,
) -> Result<PathBuf> {
    let path = run_artifacts_dir.join("stage_events.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create stage_events dir")?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("open stage_events.jsonl")?;
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(path)
}

pub fn write_progress_event_jsonl(
    run_artifacts_dir: &Path,
    event: &ProgressEventV1,
) -> Result<PathBuf> {
    let path = run_artifacts_dir.join("progress.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create progress dir")?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("open progress.jsonl")?;
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(path)
}

pub fn write_runs_export_jsonl(run_artifacts_dir: &Path, row: &RunsExportRowV1) -> Result<PathBuf> {
    let dashboard_dir = run_artifacts_dir.join("dashboard");
    std::fs::create_dir_all(&dashboard_dir).context("create dashboard dir")?;
    let path = dashboard_dir.join("runs.jsonl");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("open runs.jsonl")?;
    writeln!(file, "{}", serde_json::to_string(row)?)?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_stage_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    stage_version: i32,
    tool_id: &str,
    tool_version: &str,
    metrics_path: &Path,
    effective_config_path: &Path,
    facts_row_id: Option<&str>,
    outputs: &[PathBuf],
    subreports: &[PathBuf],
    log_paths: &[PathBuf],
    warnings: &[String],
    errors: &[String],
) -> Result<PathBuf> {
    let effective_config_hash =
        crate::services::observer::hash_file_sha256(effective_config_path).ok();
    let payload = StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version,
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        metrics_path: metrics_path.display().to_string(),
        effective_config_path: effective_config_path.display().to_string(),
        effective_config_hash,
        facts_row_id: facts_row_id.map(str::to_string),
        summary: serde_json::json!({
            "outputs": outputs.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
        }),
        warnings: warnings.to_vec(),
        errors: errors.to_vec(),
        outputs: outputs.iter().map(|p| p.display().to_string()).collect(),
        subreports: subreports.iter().map(|p| p.display().to_string()).collect(),
        log_paths: log_paths.iter().map(|p| p.display().to_string()).collect(),
    };
    let path = run_artifacts_dir.join("stage_report.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write stage_report.json")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_retention_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    tool_version: &str,
    condition: &serde_json::Value,
    parameters_json: &serde_json::Value,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.retention.json");
    let payload = RetentionReportV1 {
        schema_version: "bijux.retention_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        boundary: "pre/post".to_string(),
        numerator: serde_json::json!({
            "reads_out": reads_out,
            "bases_out": bases_out,
        }),
        denominator: serde_json::json!({
            "reads_in": reads_in,
            "bases_in": bases_in,
        }),
        scope: "reads+bases".to_string(),
        condition: condition.clone(),
        parameters_json: parameters_json.clone(),
        retention: Some(bijux_core::RetentionReportMetricV1 {
            #[allow(clippy::cast_precision_loss)]
            value: if reads_in > 0 {
                (reads_out as f64) / (reads_in as f64)
            } else {
                0.0
            },
            numerator_reads: reads_out,
            denominator_reads: reads_in,
            numerator_bases: bases_out,
            denominator_bases: bases_in,
            definition: "reads_out / reads_in".to_string(),
            stage_boundary: stage_id.to_string(),
            conditions: serde_json::json!({
                "condition": condition.clone(),
                "parameters": parameters_json.clone(),
            }),
        }),
    };
    let path = reports_dir.join(file_name);
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
        .context("write retention report")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_trim_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    adapter_preset: Option<String>,
    adapter_bank_id: Option<String>,
    adapter_bank_hash: Option<String>,
    adapter_overrides: Option<serde_json::Value>,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.trim_report.json");
    let payload = TrimReportV1 {
        schema_version: "bijux.trim_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        bases_trimmed: bases_in.saturating_sub(bases_out),
        per_adapter_counts: std::collections::BTreeMap::new(),
        adapter_preset,
        adapter_bank_id,
        adapter_bank_hash,
        adapter_overrides,
    };
    let path = reports_dir.join(file_name);
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?).context("write trim report")?;
    Ok(path)
}

pub fn write_validate_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_total: u64,
    reads_valid: u64,
    reads_invalid: u64,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.validate_report.json");
    let payload = ValidateReportV1 {
        schema_version: "bijux.validate_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_total,
        reads_valid,
        reads_invalid,
        integrity_ok: reads_invalid == 0,
    };
    let path = reports_dir.join(file_name);
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?).context("write validate report")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_merge_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_r1: u64,
    reads_r2: u64,
    reads_merged: u64,
    reads_unmerged: u64,
    merge_rate: f64,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.merge_report.json");
    let payload = MergeReportV1 {
        schema_version: "bijux.merge_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        merge_rate,
    };
    let path = reports_dir.join(file_name);
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?).context("write merge report")?;
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
    tool_invocation_path: &Path,
    metrics_envelope_path: &Path,
    stage_metrics_path: &Path,
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
            "name": "tool_invocation",
            "path": tool_invocation_path,
        }),
        serde_json::json!({
            "name": "metrics_envelope",
            "path": metrics_envelope_path,
        }),
        serde_json::json!({
            "name": "stage_metrics",
            "path": stage_metrics_path,
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
