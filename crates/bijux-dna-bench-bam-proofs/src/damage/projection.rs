use super::declarations::REQUIRED_NORMALIZED_METRIC_NAMES;
use super::declarations::REQUIRED_PARSER_OUTPUT_FIELDS;
use super::declarations::REQUIRED_SUMMARY_METRIC_NAMES;
use super::declarations::REQUIRED_UNIFIED_METRIC_NAMES;
use super::models::BamDamageCompleteRow;
use super::observations::DamageStageProof;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
// Projection consumes admitted observations; independent flags retain diagnostics.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct RowProjection<'a> {
    pub damage_row: BamDamageAuthenticityReadyRow,
    pub proof: &'a DamageStageProof,
    pub parsed_tool_ids: Vec<String>,
    pub summary_ready: bool,
    pub unified_metrics_ready: bool,
    pub parser_output_contract_ready: bool,
    pub normalized_metrics_ready: bool,
    pub damage_metric_consistency_ready: bool,
    pub missing_surfaces: Vec<String>,
    pub coverage_status: String,
    pub reason: String,
}
pub(super) fn project(
    RowProjection {
        damage_row,
        proof,
        parsed_tool_ids,
        summary_ready,
        unified_metrics_ready,
        parser_output_contract_ready,
        normalized_metrics_ready,
        damage_metric_consistency_ready,
        missing_surfaces,
        coverage_status,
        reason,
    }: RowProjection<'_>,
) -> BamDamageCompleteRow {
    BamDamageCompleteRow {
        result_id: damage_row.result_id,
        stage_id: damage_row.stage_id,
        tool_id: damage_row.tool_id,
        sample_scope: damage_row.sample_scope,
        benchmark_status: damage_row.benchmark_status,
        support_status: damage_row.support_status,
        adapter_status: damage_row.adapter_status,
        parser_status: damage_row.parser_status,
        corpus_status: damage_row.corpus_status,
        report_section_id: damage_row.report_section_id,
        summary_table_id: damage_row.summary_table_id,
        command_readiness_kind: damage_row.command_readiness_kind,
        required_output_ids: damage_row.required_output_ids,
        stage_output_ids: damage_row.stage_output_ids,
        expected_schema_extension_id: damage_row.expected_schema_extension_id,
        schema_extension_id: damage_row.schema_extension_id,
        required_schema_keys: damage_row.required_schema_keys,
        schema_required_keys: damage_row.schema_required_keys,
        required_local_smoke_fields: damage_row.required_local_smoke_fields,
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_unified_metric_names: REQUIRED_UNIFIED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_parser_output_fields: REQUIRED_PARSER_OUTPUT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        active_scope_proof_path: damage_row.active_scope_proof_path,
        command_proof_path: damage_row.command_proof_path,
        output_contract_proof_path: damage_row.output_contract_proof_path,
        parser_proof_path: damage_row.parser_proof_path,
        expected_result_proof_path: damage_row.expected_result_proof_path,
        report_map_proof_path: damage_row.report_map_proof_path,
        schema_proof_path: damage_row.schema_proof_path,
        local_smoke_proof_path: damage_row.local_smoke_proof_path,
        damage_report_path: proof.local_smoke_report.damage_report.clone(),
        terminal_position_metrics_path: proof.local_smoke_report.terminal_position_metrics.clone(),
        parser_output_path: proof.local_smoke_report.parser_output.clone(),
        advisory_boundary_path: proof.local_smoke_report.advisory_boundary.clone(),
        udg_regime_path: proof.local_smoke_report.udg_regime.clone(),
        stage_metrics_path: proof.local_smoke_report.stage_metrics.clone(),
        local_smoke_schema_version: proof.local_smoke_report.schema_version.clone(),
        local_smoke_sample_id: proof.local_smoke_report.sample_id.clone(),
        local_smoke_method: proof.local_smoke_report.method.clone(),
        local_smoke_input_bam: proof.local_smoke_report.input_bam.clone(),
        local_smoke_expectation_matched: proof.local_smoke_report.expectation_matched,
        local_smoke_tools_seen: proof.local_smoke_report.tools_seen.clone(),
        local_smoke_terminal_c_to_t_5p: proof.local_smoke_report.terminal_c_to_t_5p,
        local_smoke_terminal_g_to_a_3p: proof.local_smoke_report.terminal_g_to_a_3p,
        local_smoke_short_fragment_fraction: proof.local_smoke_report.short_fragment_fraction,
        local_smoke_damage_signal: proof.local_smoke_report.damage_signal.clone(),
        local_smoke_strict_profile_upgraded: proof.local_smoke_report.strict_profile_upgraded,
        summary_schema_version: proof.summary.schema_version.clone(),
        summary_terminal_c_to_t_5p: proof.summary.terminal_c_to_t_5p,
        summary_terminal_g_to_a_3p: proof.summary.terminal_g_to_a_3p,
        summary_short_fragment_fraction: proof.summary.short_fragment_fraction,
        summary_damage_signal: proof.summary.damage_signal.clone(),
        summary_strict_profile_upgraded: proof.summary.strict_profile_upgraded,
        summary_notes: proof.summary.notes.clone(),
        unified_canonical_metrics: proof.unified_metrics.canonical.clone(),
        unified_tools_seen: proof.unified_metrics.tools_seen.clone(),
        unified_comparison: proof.unified_metrics.comparison.clone(),
        parser_output_schema_version: proof.parser_output.schema_version.clone(),
        parsed_tool_ids,
        parsed_tools: proof.parser_output.parsed_tools.clone(),
        normalized_metrics_schema_version: proof.normalized_metrics.schema_version.clone(),
        normalized_metrics_sample_id: proof.normalized_metrics.sample_id.clone(),
        normalized_metrics_tool_id: proof.normalized_metrics.tool_id.clone(),
        normalized_metrics: proof.normalized_metrics.clone(),
        active_scope_ready: damage_row.active_scope_ready,
        command_ready: damage_row.command_ready,
        output_ready: damage_row.output_ready,
        parser_ready: damage_row.parser_ready,
        expected_result_ready: damage_row.expected_result_ready,
        report_ready: damage_row.report_ready,
        schema_ready: damage_row.schema_ready,
        local_smoke_ready: damage_row.local_smoke_ready,
        summary_ready,
        unified_metrics_ready,
        parser_output_contract_ready,
        normalized_metrics_ready,
        damage_metric_consistency_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}
