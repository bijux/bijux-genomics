use super::declarations::BamStageSpec;
use super::declarations::EXPECTED_SCHEMA_REQUIRED_KEYS;
use super::models::BamInsertSizeGcBiasReadyRow;
use super::observations::LocalSmokeProof;
use crate::command_coverage::BamCommandAdapterCoverageRow;
use crate::output_contract::BamAdapterOutputContractRow;
use crate::readiness::schema::BamStageSchemaContract;
use crate::readiness::ReadinessContext;
use crate::report_map::BamReportMapRow;
use bijux_dna_bench_bindings::expected::ExpectedBenchmarkResultRow;
use bijux_dna_bench_bindings::serving::ToolServingMapRow;
#[derive(Clone, Copy)]
pub(in crate::readiness) struct RowInput<'a> {
    pub active_row: &'a ToolServingMapRow,
    pub command_row: &'a BamCommandAdapterCoverageRow,
    pub output_row: &'a BamAdapterOutputContractRow,
    pub parser_row: &'a crate::parser_coverage::BamParserFixtureCoverageRow,
    pub expected_row: &'a ExpectedBenchmarkResultRow,
    pub report_map_row: &'a BamReportMapRow,
    pub schema_contract: &'a BamStageSchemaContract,
    pub local_smoke: &'a LocalSmokeProof,
    pub spec: BamStageSpec,
}
use super::checks::{evaluate, SurfaceChecks};
use super::explanation::{explain, Explanation};
use crate::readiness::values::owned_strings;
pub(in crate::readiness) fn build_bam_insert_size_gc_bias_ready_row(
    context: &ReadinessContext<'_>,
    input: &RowInput<'_>,
) -> BamInsertSizeGcBiasReadyRow {
    let checks = evaluate(input);
    let Explanation { missing_surfaces, coverage_status, reason } = explain(input, checks);
    let SurfaceChecks {
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        local_smoke_ready,
    } = checks;

    let RowInput {
        active_row,
        output_row,
        expected_row,
        report_map_row,
        schema_contract,
        local_smoke,
        spec,
        ..
    } = *input;
    BamInsertSizeGcBiasReadyRow {
        result_id: expected_row.result_row_id.clone(),
        stage_id: spec.stage_id.to_string(),
        tool_id: spec.tool_id.to_string(),
        sample_scope: expected_row.sample_scope.clone(),
        benchmark_status: "benchmark_ready".to_string(),
        support_status: active_row.support_status.clone(),
        adapter_status: active_row.adapter_status.clone(),
        parser_status: active_row.parser_status.clone(),
        corpus_status: active_row.corpus_status.clone(),
        report_section_id: report_map_row.report_section_id.clone(),
        summary_table_id: report_map_row.summary_table_id.clone(),
        command_readiness_kind: expected_row.readiness_kind.clone(),
        required_output_ids: spec
            .required_output_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        stage_output_ids: output_row.stage_output_ids.clone(),
        expected_normalized_metrics_output_id: spec.normalized_metrics_output_id.to_string(),
        normalized_metrics_output_id: output_row.normalized_metrics_output_id.clone(),
        expected_schema_extension_id: spec.schema_extension_id.to_string(),
        schema_extension_id: schema_contract.extension_id.clone(),
        required_schema_keys: owned_strings(&EXPECTED_SCHEMA_REQUIRED_KEYS),
        schema_required_keys: schema_contract.required_keys.clone(),
        required_local_smoke_fields: spec
            .required_local_smoke_fields
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        local_smoke_sample_id: local_smoke.sample_id.clone(),
        local_smoke_artifact_paths: local_smoke.artifact_paths.clone(),
        local_smoke_observed_fields: local_smoke.observed_fields.clone(),
        active_scope_proof_path: context.paths.serving.to_string(),
        command_proof_path: context.paths.commands.to_string(),
        output_contract_proof_path: context.paths.outputs.to_string(),
        parser_proof_path: context.paths.parsers.to_string(),
        expected_result_proof_path: context.paths.expected.to_string(),
        report_map_proof_path: context.paths.report_map.to_string(),
        schema_proof_path: context.paths.schema.to_string(),
        local_smoke_proof_path: local_smoke.proof_path.clone(),
        active_scope_ready,
        command_ready,
        output_ready,
        parser_ready,
        expected_result_ready,
        report_ready,
        schema_ready,
        local_smoke_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}
