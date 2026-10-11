use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::evaluation_inputs::RowAssessment;
use super::evaluation_inputs::RowInputs;
use super::models::FastqValidateReadsReadyRow;
use crate::proofs::inputs::ValidationContext;
use crate::readiness::output_contract::DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH;
use crate::readiness::parsers::DEFAULT_FASTQ_PARSER_COVERAGE_PATH;
use crate::readiness::report_map::DEFAULT_FASTQ_REPORT_MAP_PATH;
pub(super) fn project(
    context: &ValidationContext<'_>,
    inputs: RowInputs<'_>,
    assessment: RowAssessment,
) -> FastqValidateReadsReadyRow {
    let RowInputs {
        active_row,
        output_row,
        parser_row,
        expected_row,
        command_row,
        comparable_row,
        report_map_row,
        smoke,
    } = inputs;
    let RowAssessment {
        coverage_status,
        validation_report_fields,
        validated_reads_manifest_fields,
        command_ready,
        output_ready,
        parser_ready,
        comparable_metrics_ready,
        expected_result_ready,
        report_ready,
        missing_surfaces,
    } = assessment;
    FastqValidateReadsReadyRow {
        result_id: expected_row.result_row_id.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id.clone(),
        sample_scope: expected_row.sample_scope.clone(),
        support_status: active_row.support_status.clone(),
        adapter_status: active_row.adapter_status.clone(),
        parser_status: parser_row.parser_status.clone(),
        corpus_status: active_row.corpus_status.clone(),
        report_section_id: report_map_row.report_section_id.clone(),
        summary_table_id: report_map_row.summary_table_id.clone(),
        command_readiness_kind: command_row.readiness_kind.clone(),
        expected_outputs: expected_row.expected_output_artifact_ids.clone(),
        raw_output_artifact_ids: output_row.raw_output_artifact_ids.clone(),
        normalized_metrics_output_id: output_row.normalized_metrics_output_id.clone(),
        shared_metric_fields: comparable_row.shared_metric_fields.clone(),
        validation_report_fields,
        validated_reads_manifest_fields,
        command_argv_output_path: context.paths.command_argv.to_string(),
        output_contract_proof_path: DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH.to_string(),
        parser_proof_path: DEFAULT_FASTQ_PARSER_COVERAGE_PATH.to_string(),
        comparable_metrics_proof_path: context.paths.comparable_metrics.to_string(),
        expected_result_proof_path: context.paths.expected_results.to_string(),
        report_map_proof_path: DEFAULT_FASTQ_REPORT_MAP_PATH.to_string(),
        smoke_proof_path: smoke.proof_path.clone(),
        smoke_sample_ids: smoke.sample_ids.clone(),
        smoke_layouts: smoke.layouts.clone(),
        smoke_read_count_totals: smoke.read_count_totals.clone(),
        smoke_validation_statuses: smoke.validation_statuses.clone(),
        smoke_failure_classes: smoke.failure_classes.clone(),
        command_ready,
        output_ready,
        parser_ready,
        comparable_metrics_ready,
        expected_result_ready,
        report_ready,
        smoke_parseable: smoke.parseable,
        sample_id_normalized: smoke.sample_id_normalized,
        layout_normalized: smoke.layout_normalized,
        read_count_normalized: smoke.read_count_normalized,
        validation_status_normalized: smoke.validation_status_normalized,
        failure_reason_normalized: smoke.failure_reason_normalized,
        coverage_status: coverage_status.clone(),
        missing_surfaces,
        reason: if coverage_status == COVERAGE_STATUS_COMPLETE {
            format!(
                "retained FASTQ validator `{}` keeps active scope, command, output, parser, expected-result, report, and normalized validation proof for `{}`",
                active_row.tool_id, active_row.stage_id
            )
        } else {
            format!(
                "retained FASTQ validator `{}` is missing at least one governed readiness or normalization surface for `{}`",
                active_row.tool_id, active_row.stage_id
            )
        },
    }
}
