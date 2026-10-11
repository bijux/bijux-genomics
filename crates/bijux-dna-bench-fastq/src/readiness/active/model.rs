use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqActiveStageToolMatrixRow {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub benchmark_status: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
    pub scope_proof_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqActiveStageToolMatrixReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub retained_row_count: usize,
    pub retained_stage_count: usize,
    pub retained_tool_count: usize,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub removed_row_count: usize,
    pub removed_stage_count: usize,
    pub removed_tool_count: usize,
    pub removed_from_scope_path: &'static str,
    pub support_status_counts: BTreeMap<String, usize>,
    pub parser_status_counts: BTreeMap<String, usize>,
    pub corpus_status_counts: BTreeMap<String, usize>,
    pub rows: Vec<FastqActiveStageToolMatrixRow>,
}

#[derive(Debug, Clone)]
pub struct FastqActiveStageToolMatrixCollection {
    pub retained_row_count: usize,
    pub retained_stage_count: usize,
    pub retained_tool_count: usize,
    pub removed_row_count: usize,
    pub removed_stage_count: usize,
    pub removed_tool_count: usize,
    pub rows: Vec<FastqActiveStageToolMatrixRow>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}
