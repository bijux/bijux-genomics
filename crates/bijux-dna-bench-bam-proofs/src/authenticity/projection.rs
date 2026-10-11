use super::declarations::REQUIRED_EVIDENCE_METRIC_IDS;
use super::declarations::REQUIRED_NORMALIZED_METRIC_NAMES;
use super::declarations::REQUIRED_REPORT_FIELDS;
use super::declarations::REQUIRED_SUMMARY_METRIC_NAMES;
use super::models::BamAuthenticityCompleteRow;
use super::observations::AuthenticityStageProof;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
// Projection consumes admitted observations; independent flags retain diagnostics.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct RowProjection<'a> {
    pub authenticity_row: BamDamageAuthenticityReadyRow,
    pub proof: &'a AuthenticityStageProof,
    pub authenticity_report_contract_ready: bool,
    pub summary_ready: bool,
    pub composition_ready: bool,
    pub normalized_metrics_ready: bool,
    pub evidence_consumption_ready: bool,
    pub missing_surfaces: Vec<String>,
    pub coverage_status: String,
    pub reason: String,
}
pub(super) fn project(input: RowProjection<'_>) -> BamAuthenticityCompleteRow {
    let RowProjection {
        authenticity_row,
        proof,
        authenticity_report_contract_ready,
        summary_ready,
        composition_ready,
        normalized_metrics_ready,
        evidence_consumption_ready,
        missing_surfaces,
        coverage_status,
        reason,
    } = input;
    BamAuthenticityCompleteRow {
        result_id: authenticity_row.result_id,
        stage_id: authenticity_row.stage_id,
        tool_id: authenticity_row.tool_id,
        sample_scope: authenticity_row.sample_scope,
        benchmark_status: authenticity_row.benchmark_status,
        support_status: authenticity_row.support_status,
        adapter_status: authenticity_row.adapter_status,
        parser_status: authenticity_row.parser_status,
        corpus_status: authenticity_row.corpus_status,
        report_section_id: authenticity_row.report_section_id,
        summary_table_id: authenticity_row.summary_table_id,
        command_readiness_kind: authenticity_row.command_readiness_kind,
        required_output_ids: authenticity_row.required_output_ids,
        stage_output_ids: authenticity_row.stage_output_ids,
        expected_schema_extension_id: authenticity_row.expected_schema_extension_id,
        schema_extension_id: authenticity_row.schema_extension_id,
        required_schema_keys: authenticity_row.required_schema_keys,
        schema_required_keys: authenticity_row.schema_required_keys,
        required_local_smoke_fields: authenticity_row.required_local_smoke_fields,
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_evidence_metric_ids: REQUIRED_EVIDENCE_METRIC_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_report_fields: REQUIRED_REPORT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        active_scope_proof_path: authenticity_row.active_scope_proof_path,
        command_proof_path: authenticity_row.command_proof_path,
        output_contract_proof_path: authenticity_row.output_contract_proof_path,
        parser_proof_path: authenticity_row.parser_proof_path,
        expected_result_proof_path: authenticity_row.expected_result_proof_path,
        report_map_proof_path: authenticity_row.report_map_proof_path,
        schema_proof_path: authenticity_row.schema_proof_path,
        local_smoke_proof_path: authenticity_row.local_smoke_proof_path,
        authenticity_report_path: proof.local_smoke_report.authenticity_report.clone(),
        authenticity_summary_path: proof.local_smoke_report.authenticity_summary.clone(),
        authenticity_composite_path: proof.local_smoke_report.authenticity_composite.clone(),
        advisory_boundary_path: proof.local_smoke_report.advisory_boundary.clone(),
        stage_metrics_path: proof.local_smoke_report.stage_metrics.clone(),
        damage_unified_metrics_path: proof.local_smoke_report.damage_unified_metrics.clone(),
        contamination_summary_path: proof.local_smoke_report.contamination_summary.clone(),
        complexity_summary_path: proof.local_smoke_report.complexity_summary.clone(),
        coverage_regime_path: proof.local_smoke_report.coverage_regime.clone(),
        mapping_summary_path: proof.local_smoke_report.mapping_summary.clone(),
        local_smoke_schema_version: proof.local_smoke_report.schema_version.clone(),
        authenticity_report_schema_version: proof
            .authenticity_report
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        summary_schema_version: proof.summary.schema_version.clone(),
        composition_schema_version: proof.composition.schema_version.clone(),
        normalized_metrics_schema_version: proof.normalized_metrics.schema_version.clone(),
        local_smoke_report: proof.local_smoke_report.clone(),
        summary: proof.summary.clone(),
        composition: proof.composition.clone(),
        normalized_metrics: proof.normalized_metrics.clone(),
        active_scope_ready: authenticity_row.active_scope_ready,
        command_ready: authenticity_row.command_ready,
        output_ready: authenticity_row.output_ready,
        parser_ready: authenticity_row.parser_ready,
        expected_result_ready: authenticity_row.expected_result_ready,
        report_ready: authenticity_row.report_ready,
        schema_ready: authenticity_row.schema_ready,
        local_smoke_ready: authenticity_row.local_smoke_ready,
        authenticity_report_contract_ready,
        summary_ready,
        composition_ready,
        normalized_metrics_ready,
        evidence_consumption_ready,
        coverage_status,
        missing_surfaces,
        reason,
    }
}
