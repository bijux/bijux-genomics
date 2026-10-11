use super::models::{BamAdapterOutputContractRow, BamAdapterOutputContractStatus};
use crate::observations::ServingBinding;
use crate::EvidenceContext;
pub(super) fn render_output_contract_row(
    context: &EvidenceContext<'_>,
    row: &ServingBinding,
) -> BamAdapterOutputContractRow {
    let repo_root = context.repo_root;
    let stage_output_ids = context.output.stage_output_ids(repo_root, &row.stage_id);
    if !row_has_adapter(&row.adapter_status) {
        return BamAdapterOutputContractRow {
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            adapter_status: row.adapter_status.clone(),
            output_contract_status: BamAdapterOutputContractStatus::MissingAdapter,
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
                "row `{}` / `{}` has no runnable or plannable BAM adapter (`{}`)",
                row.stage_id, row.tool_id, row.adapter_status
            ),
        };
    }

    match context.output.assess_output(repo_root, &row.stage_id, &row.tool_id) {
        Ok(assessment) => {
            let complete = assessment.complete;
            let declared_output_ids = assessment.declared_output_ids;
            let execution_expected_output_ids = assessment.execution_expected_output_ids;
            let stage_expected_artifact_ids = assessment.stage_expected_artifact_ids;
            let normalized_metrics_output_id = assessment.normalized_metrics_output_id;
            let raw_output_artifact_ids = assessment.raw_output_artifact_ids;
            let missing_declarations = assessment.missing_declarations;
            let output_contract_status = if complete {
                BamAdapterOutputContractStatus::Complete
            } else {
                BamAdapterOutputContractStatus::Incomplete
            };
            let path_template_root =
                path_template_root(context.paths.slurm_run_id, &row.stage_id, &row.tool_id);
            let stdout_path_template = Some(format!("{path_template_root}/stdout.log"));
            let stderr_path_template = Some(format!("{path_template_root}/stderr.log"));
            let stage_result_manifest_path_template =
                Some(format!("{path_template_root}/stage-result.json"));
            let reason = if complete {
                format!(
                    "row `{}` / `{}` declares all governed BAM stage outputs, a normalized metrics artifact, and deterministic stdout/stderr/result-manifest paths",
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

            BamAdapterOutputContractRow {
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
        Err(error) => BamAdapterOutputContractRow {
            tool_id: row.tool_id.clone(),
            stage_id: row.stage_id.clone(),
            adapter_status: row.adapter_status.clone(),
            output_contract_status: BamAdapterOutputContractStatus::Incomplete,
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
                "row `{}` / `{}` could not load the governed BAM output contract: {error}",
                row.stage_id, row.tool_id
            ),
        },
    }
}
pub(super) fn row_has_adapter(adapter_status: &str) -> bool {
    matches!(adapter_status, "runnable" | "plannable")
}
pub(super) fn path_template_root(run_id: &str, stage_id: &str, tool_id: &str) -> String {
    format!(
        "artifacts/runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        run_id, "{fixture_scope}", stage_id, "{sample_scope}", tool_id
    )
}
