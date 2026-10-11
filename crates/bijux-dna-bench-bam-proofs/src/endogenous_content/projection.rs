use super::declarations::OPTIONAL_METRIC_NAMES;
use super::declarations::REQUIRED_NORMALIZED_METRIC_NAMES;
use super::declarations::REQUIRED_SUMMARY_METRIC_NAMES;
use super::models::BamEndogenousContentCompleteRow;
use super::models::LocalEndogenousContentSmokeMetrics;
use super::observations::LocalEndogenousContentSmokeReport;
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyRow;
// Projection consumes admitted observations; independent flags retain diagnostics.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct RowProjection {
    pub endogenous_row: BamOverlapEndogenousReadyRow,
    pub local_smoke_report: LocalEndogenousContentSmokeReport,
    pub summary: bijux_dna_domain_bam::BamEndogenousContentEstimateV1,
    pub normalized_metrics: LocalEndogenousContentSmokeMetrics,
    pub summary_ready: bool,
    pub normalized_metrics_ready: bool,
    pub endogenous_metric_consistency_ready: bool,
    pub missing_surfaces: Vec<String>,
    pub coverage_status: String,
    pub reason: String,
}
pub(super) fn project(input: RowProjection) -> BamEndogenousContentCompleteRow {
    let RowProjection {
        endogenous_row,
        local_smoke_report,
        summary,
        normalized_metrics,
        summary_ready,
        normalized_metrics_ready,
        endogenous_metric_consistency_ready,
        missing_surfaces,
        coverage_status,
        reason,
    } = input;
    BamEndogenousContentCompleteRow {
        result_id: endogenous_row.result_id,
        stage_id: endogenous_row.stage_id,
        tool_id: endogenous_row.tool_id,
        sample_scope: endogenous_row.sample_scope,
        benchmark_status: endogenous_row.benchmark_status,
        support_status: endogenous_row.support_status,
        adapter_status: endogenous_row.adapter_status,
        parser_status: endogenous_row.parser_status,
        corpus_status: endogenous_row.corpus_status,
        report_section_id: endogenous_row.report_section_id,
        summary_table_id: endogenous_row.summary_table_id,
        command_readiness_kind: endogenous_row.command_readiness_kind,
        required_output_ids: endogenous_row.required_output_ids,
        stage_output_ids: endogenous_row.stage_output_ids,
        expected_schema_extension_id: endogenous_row.expected_schema_extension_id,
        schema_extension_id: endogenous_row.schema_extension_id,
        required_schema_keys: endogenous_row.required_schema_keys,
        schema_required_keys: endogenous_row.schema_required_keys,
        required_local_smoke_fields: endogenous_row.required_local_smoke_fields,
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        optional_metric_names: OPTIONAL_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        available_optional_metric_names: Vec::new(),
        active_scope_proof_path: endogenous_row.active_scope_proof_path,
        command_proof_path: endogenous_row.command_proof_path,
        output_contract_proof_path: endogenous_row.output_contract_proof_path,
        parser_proof_path: endogenous_row.parser_proof_path,
        expected_result_proof_path: endogenous_row.expected_result_proof_path,
        report_map_proof_path: endogenous_row.report_map_proof_path,
        schema_proof_path: endogenous_row.schema_proof_path,
        local_smoke_proof_path: endogenous_row.local_smoke_proof_path,
        endogenous_report_path: local_smoke_report.endogenous_report.clone(),
        endogenous_summary_path: local_smoke_report.endogenous_summary.clone(),
        stage_metrics_path: local_smoke_report.stage_metrics.clone(),
        local_smoke_schema_version: local_smoke_report.schema_version,
        local_smoke_sample_id: local_smoke_report.sample_id.clone(),
        local_smoke_method: local_smoke_report.method.clone(),
        local_smoke_input_bam: local_smoke_report.input_bam,
        local_smoke_expectation_matched: local_smoke_report.expectation_matched,
        local_smoke_host_reference_scope: local_smoke_report.host_reference_scope.clone(),
        local_smoke_mapped_reads: local_smoke_report.mapped_reads,
        local_smoke_endogenous_reads: local_smoke_report.endogenous_reads,
        local_smoke_total_reads: local_smoke_report.total_reads,
        local_smoke_endogenous_fraction: local_smoke_report.endogenous_fraction,
        local_smoke_prealignment_fraction: local_smoke_report.prealignment_fraction,
        summary_schema_version: summary.schema_version,
        summary_method: summary.method,
        summary_host_reference_scope: summary.host_reference_scope,
        summary_mapped_reads: summary.mapped_reads,
        summary_endogenous_reads: summary.endogenous_reads,
        summary_total_reads: summary.total_reads,
        summary_endogenous_fraction: summary.endogenous_fraction,
        summary_prealignment_fraction: summary.prealignment_fraction,
        summary_postalignment_fraction: summary.postalignment_fraction,
        summary_caveats: summary.caveats,
        contaminant_reads: None,
        normalized_metrics_schema_version: normalized_metrics.schema_version.clone(),
        normalized_metrics_sample_id: normalized_metrics.sample_id.clone(),
        normalized_metrics,
        active_scope_ready: endogenous_row.active_scope_ready,
        command_ready: endogenous_row.command_ready,
        output_ready: endogenous_row.output_ready,
        parser_ready: endogenous_row.parser_ready,
        expected_result_ready: endogenous_row.expected_result_ready,
        report_ready: endogenous_row.report_ready,
        schema_ready: endogenous_row.schema_ready,
        local_smoke_ready: endogenous_row.local_smoke_ready,
        summary_ready,
        normalized_metrics_ready,
        endogenous_metric_consistency_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}
