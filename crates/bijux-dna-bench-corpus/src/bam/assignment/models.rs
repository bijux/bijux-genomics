use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamCorpusAssignmentRow {
    pub tool_id: String,
    pub stage_id: String,
    pub benchmark_status: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub sample_id: String,
    pub input_contract: String,
    pub benchmark_limits: String,
    pub required_assets: String,
    pub expected_outputs: String,
    pub skip_behavior: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct BamCorpusAssignmentReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub benchmark_ready_row_count: usize,
    pub corpus_family_counts: BTreeMap<String, usize>,
    pub fixture_counts: BTreeMap<String, usize>,
    pub rows: Vec<BamCorpusAssignmentRow>,
}
