use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfActiveStageToolMatrixRow {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_status: String,
    pub stage_support_status: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub scope_state: String,
    pub scope_detail: String,
    pub scope_proof_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfActiveStageToolMatrixReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub active_row_count: usize,
    pub complete_row_count: usize,
    pub removed_row_count: usize,
    pub scope_state_counts: BTreeMap<String, usize>,
    pub scope_detail_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfActiveStageToolMatrixRow>,
}
