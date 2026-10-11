use serde::Serialize;
pub const DEFAULT_VCF_ALL_RETAINED_TOOLS_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/VCF_ALL_RETAINED_TOOLS_COMPLETE.json";
pub(super) const VCF_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_all_retained_tools_complete.v1";
pub(super) const EXPECTED_CHECKED_GOAL_COUNT: usize = 24;
pub(super) const EXPECTED_RETAINED_ROW_COUNT: usize = 39;
pub(super) const EXPECTED_RETAINED_STAGE_COUNT: usize = 20;
pub(super) const EXPECTED_RETAINED_TOOL_COUNT: usize = 16;
pub(super) const EXPECTED_ACTIVE_STAGE_COUNT: usize = 20;
pub(super) const EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT: usize = 20;
pub(super) const EXPECTED_CONTAINER_SMOKE_ROW_COUNT: usize = 15;
pub(super) const EXPECTED_EXTERNAL_ARTIFACT_ROW_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct VcfBindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAllRetainedToolsCompleteGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfAllRetainedToolsCompleteReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub retained_row_count: usize,
    pub retained_stage_count: usize,
    pub retained_tool_count: usize,
    pub active_row_count: usize,
    pub removed_row_count: usize,
    pub active_stage_count: usize,
    pub active_tool_count: usize,
    pub expected_result_row_count: usize,
    pub rendered_command_row_count: usize,
    pub parser_fixture_row_count: usize,
    pub local_smoke_row_count: usize,
    pub local_smoke_host_stage_row_count: usize,
    pub local_smoke_container_row_count: usize,
    pub local_smoke_external_artifact_row_count: usize,
    pub report_map_row_count: usize,
    pub ok: bool,
    pub checks: Vec<VcfAllRetainedToolsCompleteGoalCheck>,
}
