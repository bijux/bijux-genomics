use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy)]
pub(super) struct TrimStageSpec {
    pub(super) stage_id: &'static str,
    pub(super) expected_tool_ids: &'static [&'static str],
    pub(super) required_metric_fields: &'static [&'static str],
}
#[derive(Debug, Clone, Serialize)]
// Each flag records an independent evidence condition.
#[allow(clippy::struct_excessive_bools)]
pub struct FastqTrimStagesReadyRow {
    pub result_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub sample_scope: String,
    pub benchmark_status: String,
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
    pub schema_extension_id: String,
    pub required_metric_fields: Vec<String>,
    pub schema_required_fields: Vec<String>,
    pub active_scope_proof_path: String,
    pub command_proof_path: String,
    pub output_contract_proof_path: String,
    pub parser_proof_path: String,
    pub expected_result_proof_path: String,
    pub report_map_proof_path: String,
    pub schema_proof_path: String,
    pub active_scope_ready: bool,
    pub command_ready: bool,
    pub output_ready: bool,
    pub parser_ready: bool,
    pub expected_result_ready: bool,
    pub report_ready: bool,
    pub schema_ready: bool,
    pub coverage_status: String,
    pub missing_surfaces: Vec<String>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct FastqTrimStagesReadyReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub active_row_count: usize,
    pub complete_row_count: usize,
    pub incomplete_row_count: usize,
    pub checked_surface_count: usize,
    pub stage_count: usize,
    pub expected_tool_ids_by_stage: BTreeMap<String, Vec<String>>,
    pub required_metric_fields_by_stage: BTreeMap<String, Vec<String>>,
    pub coverage_status_counts: BTreeMap<String, usize>,
    pub violation_count: usize,
    pub ok: bool,
    pub rows: Vec<FastqTrimStagesReadyRow>,
    pub violations: Vec<FastqTrimStagesReadyRow>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}
#[derive(Debug, Clone)]
pub(super) struct TrimStageSchemaContract {
    pub(super) extension_id: String,
    pub(super) required_fields: Vec<String>,
}
