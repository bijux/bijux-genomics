use super::models::FastqAdapterOutputContractRow;
use super::models::FastqAdapterOutputContractStatus;
use super::paths::path_template_root;
use crate::readiness::declarations::observations::ServingBinding as ToolServingMapRow;
use crate::readiness::declarations::OutputContext;
use anyhow::Result;
use bijux_dna_domain_fastq::stages::ports::stage_output_ids_in_manifest_order;
/// Collect canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn collect_fastq_adapter_output_contract_rows(
    context: &OutputContext<'_>,
) -> Result<Vec<FastqAdapterOutputContractRow>> {
    let repo_root = context.repo_root;
    let tool_map = context.evidence.serving(repo_root)?;
    Ok(tool_map.iter().map(|row| render_output_contract_row(context, row)).collect::<Vec<_>>())
}

pub(super) fn render_output_contract_row(
    context: &OutputContext<'_>,
    row: &ToolServingMapRow,
) -> FastqAdapterOutputContractRow {
    let repo_root = context.repo_root;
    let stage_output_ids = stage_output_ids_in_manifest_order(&row.stage_id).unwrap_or_default();
    if !row_has_adapter(&row.adapter_status) {
        return FastqAdapterOutputContractRow {
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            adapter_status: row.adapter_status.clone(),
            output_contract_status: FastqAdapterOutputContractStatus::MissingAdapter,
            stage_output_ids,
            stage_expected_artifact_ids: Vec::new(),
            declared_output_ids: Vec::new(),
            execution_expected_output_ids: Vec::new(),
            missing_declarations: vec!["adapter".to_string()],
            raw_output_artifact_ids: Vec::new(),
            normalized_metrics_output_id: None,
            stdout_path_template: None,
            stderr_path_template: None,
            stage_result_manifest_path_template: None,
            reason: format!(
                "row `{}` / `{}` has no runnable or plannable FASTQ adapter (`{}`)",
                row.stage_id, row.tool_id, row.adapter_status
            ),
        };
    }

    match context.evidence.assess(repo_root, &row.stage_id, &row.tool_id) {
        Ok((contract, assessment)) => {
            let declared_output_ids = contract.declared_output_ids;
            let execution_expected_output_ids = contract.execution_expected_output_ids;
            let stage_expected_artifact_ids = contract.stage_expected_artifact_ids;
            let complete = assessment.is_complete();
            let normalized_metrics_output_id = assessment.normalized_metrics_output_id;
            let raw_output_artifact_ids = assessment.raw_output_artifact_ids;
            let missing_declarations = assessment.missing_declarations;
            let output_contract_status = if complete {
                FastqAdapterOutputContractStatus::Complete
            } else {
                FastqAdapterOutputContractStatus::Incomplete
            };
            let path_template_root = path_template_root(context, &row.stage_id, &row.tool_id);
            let stdout_path_template = Some(format!("{path_template_root}/stdout.log"));
            let stderr_path_template = Some(format!("{path_template_root}/stderr.log"));
            let stage_result_manifest_path_template =
                Some(format!("{path_template_root}/stage-result.json"));
            let reason = if complete {
                format!(
                    "row `{}` / `{}` declares all governed FASTQ stage outputs, a normalized metrics artifact, and deterministic stdout/stderr/result-manifest paths",
                    row.stage_id, row.tool_id
                )
            } else {
                format!(
                    "row `{}` / `{}` is missing governed adapter output declarations: {}",
                    row.stage_id,
                    row.tool_id,
                    missing_declarations.join(", ")
                )
            };

            FastqAdapterOutputContractRow {
                tool_id: row.tool_id.clone(),
                stage_id: row.stage_id.clone(),
                adapter_status: row.adapter_status.clone(),
                output_contract_status,
                stage_output_ids,
                stage_expected_artifact_ids,
                declared_output_ids,
                execution_expected_output_ids,
                missing_declarations,
                raw_output_artifact_ids,
                normalized_metrics_output_id,
                stdout_path_template,
                stderr_path_template,
                stage_result_manifest_path_template,
                reason,
            }
        }
        Err(error) => FastqAdapterOutputContractRow {
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            adapter_status: row.adapter_status.clone(),
            output_contract_status: FastqAdapterOutputContractStatus::Incomplete,
            stage_output_ids,
            stage_expected_artifact_ids: Vec::new(),
            declared_output_ids: Vec::new(),
            execution_expected_output_ids: Vec::new(),
            missing_declarations: vec!["stage_contract".to_string()],
            raw_output_artifact_ids: Vec::new(),
            normalized_metrics_output_id: None,
            stdout_path_template: None,
            stderr_path_template: None,
            stage_result_manifest_path_template: None,
            reason: format!(
                "row `{}` / `{}` could not load the governed FASTQ output contract: {error}",
                row.stage_id, row.tool_id
            ),
        },
    }
}

pub(super) fn row_has_adapter(adapter_status: &str) -> bool {
    matches!(adapter_status, "runnable" | "plannable")
}
