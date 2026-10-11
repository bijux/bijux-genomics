use super::evaluation_inputs::RowAssessment;
use super::evaluation_inputs::RowInputs;
use super::models::FastqFilterStagesReadyRow;
use crate::proofs::inputs::SchemaContext;
use crate::readiness::adapters::DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH;
use crate::readiness::output_contract::DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH;
use crate::readiness::parsers::DEFAULT_FASTQ_PARSER_COVERAGE_PATH;
use crate::readiness::report_map::DEFAULT_FASTQ_REPORT_MAP_PATH;
pub(super) fn project(
    context: &SchemaContext<'_>,
    inputs: RowInputs<'_>,
    assessment: RowAssessment,
) -> FastqFilterStagesReadyRow {
    let RowInputs {
        active_row,
        output_row,
        expected_row,
        report_map_row,
        schema_contract,
        spec,
        schema_proof_path,
        ..
    } = inputs;
    let RowAssessment {
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        coverage_status,
        missing_surfaces,
        reason,
    } = assessment;
    FastqFilterStagesReadyRow {
        result_id: expected_row.result_row_id.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id,
        sample_scope: expected_row.sample_scope.clone(),
        benchmark_status: active_row.benchmark_status,
        support_status: active_row.support_status,
        adapter_status: active_row.adapter_status,
        parser_status: active_row.parser_status,
        corpus_status: active_row.corpus_status,
        report_section_id: report_map_row.report_section_id.clone(),
        summary_table_id: report_map_row.summary_table_id.clone(),
        command_readiness_kind: expected_row.readiness_kind.clone(),
        expected_outputs: expected_row.expected_output_artifact_ids.clone(),
        required_output_ids: spec
            .required_output_ids
            .iter()
            .map(|output_id| (*output_id).to_string())
            .collect(),
        raw_output_artifact_ids: output_row.raw_output_artifact_ids.clone(),
        normalized_metrics_output_id: output_row.normalized_metrics_output_id.clone(),
        expected_normalized_metrics_output_id: spec.normalized_metrics_output_id.to_string(),
        schema_extension_id: schema_contract.extension_id.clone(),
        required_metric_fields: spec
            .required_metric_fields
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        schema_required_fields: schema_contract.required_fields.clone(),
        active_scope_proof_path: active_row.scope_proof_path,
        command_proof_path: DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH.to_string(),
        output_contract_proof_path: DEFAULT_FASTQ_ADAPTER_OUTPUT_CONTRACT_PATH.to_string(),
        parser_proof_path: DEFAULT_FASTQ_PARSER_COVERAGE_PATH.to_string(),
        expected_result_proof_path: context.paths.expected_results.to_string(),
        report_map_proof_path: DEFAULT_FASTQ_REPORT_MAP_PATH.to_string(),
        schema_proof_path: schema_proof_path.to_string(),
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}
