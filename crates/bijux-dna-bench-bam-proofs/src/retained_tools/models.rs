use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamAllRetainedToolsCompleteGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamAllRetainedToolsCompleteReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub retained_row_count: usize,
    pub retained_stage_count: usize,
    pub retained_tool_count: usize,
    pub command_adapter_row_count: usize,
    pub output_declaration_row_count: usize,
    pub expected_result_row_count: usize,
    pub rendered_command_row_count: usize,
    pub parser_fixture_row_count: usize,
    pub local_smoke_row_count: usize,
    pub local_smoke_host_stage_row_count: usize,
    pub local_smoke_container_row_count: usize,
    pub local_smoke_external_artifact_row_count: usize,
    pub report_map_row_count: usize,
    pub active_row_consistency_surface_count: usize,
    pub micro_smoke_family_count: usize,
    pub science_threshold_stage_count: usize,
    pub ok: bool,
    pub checks: Vec<BamAllRetainedToolsCompleteGoalCheck>,
}
