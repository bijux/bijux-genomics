use super::{
    identity::{result_root_path, result_row_id},
    ExpectedBenchmarkResultRow, ProjectionContext,
};
use crate::observations::OutputContract;
use anyhow::{anyhow, Result};
pub(in crate::expected) fn build_bam_expected_result_row(
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
            "BAM benchmark-ready row `{stage_id}` / `{tool_id}` is missing a complete adapter output contract"
        ));
    }
    let sample_scope = context
        .fixture_sample_scopes
        .get(fixture_id)
        .cloned()
        .ok_or_else(|| anyhow!("missing BAM fixture sample scope for `{fixture_id}`"))?;
    let result_root =
        result_root_path(context.request, fixture_id, stage_id, &sample_scope, tool_id);
    let stage_result_manifest_path = format!("{result_root}/stage-result.json");
    let stdout_path = format!("{result_root}/stdout.log");
    let stderr_path = format!("{result_root}/stderr.log");
    let normalized_metrics_output_id = contract_row.normalized_metrics_output_id.clone();
    let expected_output_artifact_ids = contract_row.execution_expected_output_ids.clone();
    let raw_output_artifact_ids = contract_row.raw_output_artifact_ids.clone();

    Ok(ExpectedBenchmarkResultRow {
        result_row_id: result_row_id("bam", fixture_id, stage_id, &sample_scope, tool_id),
        domain: "bam".to_string(),
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
