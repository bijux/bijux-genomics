use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::metrics::{MetricEnvelope, MetricsEnvelope, StageMetricsV1, ToolInvocationV1};

use crate::StageObservabilityContextV1;

use super::io::write_canonical_json;

/// # Errors
/// Returns an error if metrics JSON cannot be written.
pub fn write_metrics_json<T: serde::Serialize>(
    run_dirs: &super::manifests::RunDirs,
    execution: &bijux_dna_core::prelude::measure::ExecutionMetrics,
    metrics: &MetricEnvelope<T>,
) -> Result<()> {
    let payload = serde_json::json!({
        "execution": execution,
        "metrics": metrics
    });
    write_canonical_json(&run_dirs.metrics_path, &payload).context("write metrics.json")?;
    Ok(())
}

/// # Errors
/// Returns an error if the metrics envelope cannot be written.
pub fn write_metrics_envelope(
    run_artifacts_dir: &Path,
    ctx: &StageObservabilityContextV1,
    metrics: &serde_json::Value,
    input_hashes: &[String],
) -> Result<PathBuf> {
    let run_id = run_artifacts_dir
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow::anyhow!("run id missing from run artifacts dir"))?
        .to_string();
    let manifest_hash = run_artifacts_dir
        .parent()
        .map(|dir| dir.join("run_manifest.json"))
        .filter(|path| path.exists())
        .and_then(|path| bijux_dna_infra::hash_file_sha256(&path).ok());
    let image_digest = ctx
        .metric_context
        .image_digest
        .clone()
        .ok_or_else(|| anyhow::anyhow!("metrics envelope requires declared image digest"))?;
    let payload: MetricsEnvelope<serde_json::Value> = MetricsEnvelope {
        schema_version: "bijux.metrics_envelope.v2".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: ctx.stage_id.clone(),
        stage_version: ctx.stage_version,
        tool_id: ctx.tool_id.clone(),
        tool_version: ctx.tool_version.clone(),
        image_digest,
        parameters_fingerprint: ctx.parameters_fingerprint.clone(),
        input_fingerprint: ctx.input_fingerprint.clone(),
        parameters_json_normalized: ctx.parameters_json_normalized.clone(),
        input_hashes: input_hashes.to_vec(),
        metric_provenance: Some(bijux_dna_core::contract::MetricProvenanceV1 {
            run_id,
            stage_id: ctx.stage_id.clone(),
            tool_id: ctx.tool_id.clone(),
            tool_version: ctx.tool_version.clone(),
            params_hash: ctx.parameters_fingerprint.clone(),
            input_artifact_hashes: if input_hashes.is_empty() {
                vec![ctx.input_fingerprint.clone()]
            } else {
                input_hashes.to_vec()
            },
            manifest_hash,
        }),
        metrics: metrics.clone(),
    };
    let path = run_artifacts_dir.join("metrics_envelope.json");
    write_canonical_json(&path, &payload).context("write metrics_envelope.json")?;
    Ok(path)
}

/// # Errors
/// Returns an error if stage metrics cannot be written.
pub fn write_stage_metrics_json<T: serde::Serialize>(
    run_artifacts_dir: &Path,
    metrics: &StageMetricsV1<T>,
) -> Result<PathBuf> {
    let stage_path = run_artifacts_dir.join("stage_metrics.json");
    let metrics_path = run_artifacts_dir.join("metrics.json");
    write_canonical_json(&stage_path, metrics).context("write stage_metrics.json")?;
    write_canonical_json(&metrics_path, metrics).context("write metrics.json")?;
    Ok(stage_path)
}

/// # Errors
/// Returns an error if tool invocation JSON cannot be written.
pub fn write_tool_invocation_json(
    run_artifacts_dir: &Path,
    stage_id: &str,
    invocation: &ToolInvocationV1,
) -> Result<PathBuf> {
    validate_file_stem("stage_id", stage_id)?;
    let invocations_dir = run_artifacts_dir.join("invocations");
    bijux_dna_infra::ensure_dir(&invocations_dir).context("create invocations dir")?;
    let file_name = format!("{stage_id}.tool_invocation.json");
    let path = invocations_dir.join(file_name);
    write_canonical_json(&path, invocation).context("write tool_invocation.json")?;
    Ok(path)
}

fn validate_file_stem(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} file stem must not be empty"));
    }
    if value.contains(std::path::MAIN_SEPARATOR)
        || value.contains('/')
        || value.contains('\\')
        || value == "."
        || value == ".."
    {
        return Err(anyhow!("{label} file stem must not contain path separators"));
    }
    Ok(())
}
