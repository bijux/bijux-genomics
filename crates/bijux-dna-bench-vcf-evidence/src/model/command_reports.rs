use super::commands::VcfRenderedCommandRow;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfRenderedCommandArgvRow {
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_steps: Vec<VcfRenderedCommandArgvStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfRenderedCommandArgvStep {
    pub step_id: String,
    pub step_kind: String,
    pub consumes_previous_stdout: bool,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfRenderedCommandsReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub argv_output_path: String,
    pub row_count: usize,
    pub rows: Vec<VcfRenderedCommandRow>,
}

/// Publication addresses supplied alongside admitted command observations.
/// These locations do not establish command or scientific readiness.
pub struct VcfCommandPublicationPaths {
    pub output_path: String,
    pub argv_output_path: String,
}
