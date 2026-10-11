#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VcfRenderedCommandStep {
    pub step_id: String,
    pub step_kind: String,
    pub consumes_previous_stdout: bool,
    pub argv: Vec<String>,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VcfRenderedCommandRow {
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_kind: String,
    pub benchmark_status: String,
    pub command_source: String,
    pub command_steps: Vec<VcfRenderedCommandStep>,
    pub script_commands: Vec<String>,
    pub reason: String,
}
