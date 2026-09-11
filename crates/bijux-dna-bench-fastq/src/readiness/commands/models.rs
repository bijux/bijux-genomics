use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqRenderedCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub consumes_previous_stdout: bool,
    pub argv: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqRenderedCommandRow {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_steps: Vec<FastqRenderedCommandStep>,
    pub script_commands: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqRenderedCommandArgvRow {
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_steps: Vec<FastqRenderedCommandStep>,
}
#[derive(Debug, Clone, Serialize)]
pub struct FastqRenderedCommandsReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub argv_output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub command_source_counts: BTreeMap<String, usize>,
    pub rows: Vec<FastqRenderedCommandRow>,
}
