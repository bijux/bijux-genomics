use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize)]
// Each flag records an independent evidence condition.
#[allow(clippy::struct_excessive_bools)]
pub struct FastqValidateReadsReadyRow {
    pub result_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub sample_scope: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub report_section_id: String,
    pub summary_table_id: String,
    pub command_readiness_kind: String,
    pub expected_outputs: Vec<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub normalized_metrics_output_id: Option<String>,
    pub shared_metric_fields: Vec<String>,
    pub validation_report_fields: Vec<String>,
    pub validated_reads_manifest_fields: Vec<String>,
    pub command_argv_output_path: String,
    pub output_contract_proof_path: String,
    pub parser_proof_path: String,
    pub comparable_metrics_proof_path: String,
    pub expected_result_proof_path: String,
    pub report_map_proof_path: String,
    pub smoke_proof_path: String,
    pub smoke_sample_ids: Vec<String>,
    pub smoke_layouts: Vec<String>,
    pub smoke_read_count_totals: Vec<u64>,
    pub smoke_validation_statuses: Vec<String>,
    pub smoke_failure_classes: Vec<String>,
    pub command_ready: bool,
    pub output_ready: bool,
    pub parser_ready: bool,
    pub comparable_metrics_ready: bool,
    pub expected_result_ready: bool,
    pub report_ready: bool,
    pub smoke_parseable: bool,
    pub sample_id_normalized: bool,
    pub layout_normalized: bool,
    pub read_count_normalized: bool,
    pub validation_status_normalized: bool,
    pub failure_reason_normalized: bool,
    pub coverage_status: String,
    pub missing_surfaces: Vec<String>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct FastqValidateReadsReadyReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub active_row_count: usize,
    pub complete_row_count: usize,
    pub incomplete_row_count: usize,
    pub checked_surface_count: usize,
    pub expected_tool_ids: Vec<String>,
    pub validation_status_field_id: &'static str,
    pub failure_reason_field_id: &'static str,
    pub required_validation_report_fields: Vec<String>,
    pub required_validated_reads_manifest_fields: Vec<String>,
    pub sample_case_count: usize,
    pub smoke_sample_ids: Vec<String>,
    pub smoke_layouts: Vec<String>,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<FastqValidateReadsReadyRow>,
    pub violations: Vec<FastqValidateReadsReadyRow>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}
#[derive(Debug, Clone)]
// Each flag records an independent evidence condition.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct FastqValidateReadsSmokeEvidence {
    pub(super) proof_path: String,
    pub(super) sample_case_count: usize,
    pub(super) sample_ids: Vec<String>,
    pub(super) layouts: Vec<String>,
    pub(super) read_count_totals: Vec<u64>,
    pub(super) validation_statuses: Vec<String>,
    pub(super) failure_classes: Vec<String>,
    pub(super) parseable: bool,
    pub(super) sample_id_normalized: bool,
    pub(super) layout_normalized: bool,
    pub(super) read_count_normalized: bool,
    pub(super) validation_status_normalized: bool,
    pub(super) failure_reason_normalized: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalValidateReadsSmokeReport {
    pub(super) cases: Vec<LocalValidateReadsSmokeCase>,
}
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalValidateReadsSmokeCase {
    pub(super) sample_id: String,
    pub(super) layout: String,
    pub(super) input_read_count_total: u64,
    pub(super) input_pair_count: Option<u64>,
    pub(super) validation_status: String,
    pub(super) validation_report: String,
    pub(super) validated_reads_manifest: String,
}
