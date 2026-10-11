use super::{
    identity::{result_root_path, result_row_id},
    ExpectedBenchmarkResultRow, ProjectionContext,
};
use crate::observations::OutputContract;
use anyhow::{anyhow, Result};
pub(in crate::expected) fn build_fastq_expected_result_row(
    stage_id: &str,
    tool_id: &str,
    readiness_kind: &str,
    corpus_family_id: &str,
    fixture_id: &str,
    context: &ProjectionContext<'_, '_>,
    contract_row: &OutputContract,
) -> Result<ExpectedBenchmarkResultRow> {
    if !contract_row.complete {
        return Err(anyhow!(
            "FASTQ benchmark-ready row `{stage_id}` / `{tool_id}` is missing a complete adapter output contract"
        ));
    }
    let sample_scope = context
        .fixture_sample_scopes
        .get(fixture_id)
        .cloned()
        .ok_or_else(|| anyhow!("missing FASTQ fixture sample scope for `{fixture_id}`"))?;
    let result_root =
        result_root_path(context.request, fixture_id, stage_id, &sample_scope, tool_id);
    let stage_result_manifest_path = format!("{result_root}/stage-result.json");
    let stdout_path = format!("{result_root}/stdout.log");
    let stderr_path = format!("{result_root}/stderr.log");
    let normalized_metrics_output_id = contract_row.normalized_metrics_output_id.clone();
    let expected_output_artifact_ids = contract_row.execution_expected_output_ids.clone();
    let raw_output_artifact_ids = contract_row.raw_output_artifact_ids.clone();

    Ok(ExpectedBenchmarkResultRow {
        result_row_id: result_row_id("fastq", fixture_id, stage_id, &sample_scope, tool_id),
        domain: "fastq".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        readiness_kind: readiness_kind.to_string(),
        corpus_family_id: corpus_family_id.to_string(),
        fixture_id: fixture_id.to_string(),
        sample_scope,
        expected_output_artifact_ids,
        raw_output_artifact_ids,
        normalized_metrics_output_id: normalized_metrics_output_id.clone(),
        result_root: result_root.clone(),
        stage_result_manifest_path: stage_result_manifest_path.clone(),
        stdout_path,
        stderr_path,
        reason: format!(
            "benchmark row `{stage_id}` / `{tool_id}` on fixture `{fixture_id}` expects `{stage_result_manifest_path}` with normalized metrics artifact `{}`",
            normalized_metrics_output_id.unwrap_or_else(|| "not_declared".to_string())
        ),
    })
}
pub(in crate::expected) fn build_fastq_asset_backed_expected_result_row(
    context: &ProjectionContext<'_, '_>,
    stage_id: &str,
    tool_id: &str,
    readiness_kind: &str,
    benchmark_scope_id: &str,
    contract_row: &OutputContract,
) -> Result<ExpectedBenchmarkResultRow> {
    if !contract_row.complete {
        return Err(anyhow!(
            "FASTQ asset-backed row `{stage_id}` / `{tool_id}` is missing a complete adapter output contract"
        ));
    }
    let sample_scope = "asset-set".to_string();
    let result_root =
        result_root_path(context.request, benchmark_scope_id, stage_id, &sample_scope, tool_id);
    let stage_result_manifest_path = format!("{result_root}/stage-result.json");
    let stdout_path = format!("{result_root}/stdout.log");
    let stderr_path = format!("{result_root}/stderr.log");
    let normalized_metrics_output_id = contract_row.normalized_metrics_output_id.clone();
    let expected_output_artifact_ids = contract_row.execution_expected_output_ids.clone();
    let raw_output_artifact_ids = contract_row.raw_output_artifact_ids.clone();

    Ok(ExpectedBenchmarkResultRow {
        result_row_id: result_row_id("fastq", benchmark_scope_id, stage_id, &sample_scope, tool_id),
        domain: "fastq".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        readiness_kind: readiness_kind.to_string(),
        corpus_family_id: "asset_backed".to_string(),
        fixture_id: benchmark_scope_id.to_string(),
        sample_scope,
        expected_output_artifact_ids,
        raw_output_artifact_ids,
        normalized_metrics_output_id: normalized_metrics_output_id.clone(),
        result_root: result_root.clone(),
        stage_result_manifest_path: stage_result_manifest_path.clone(),
        stdout_path,
        stderr_path,
        reason: format!(
            "asset-backed benchmark row `{stage_id}` / `{tool_id}` in scope `{benchmark_scope_id}` expects `{stage_result_manifest_path}` with normalized metrics artifact `{}`",
            normalized_metrics_output_id.unwrap_or_else(|| "not_declared".to_string())
        ),
    })
}
