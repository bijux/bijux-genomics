use super::{
    model::{
        VcfAllRetainedToolsCompleteGoalCheck, VcfAllRetainedToolsCompleteReport,
        EXPECTED_CHECKED_GOAL_COUNT, EXPECTED_RETAINED_ROW_COUNT,
        VCF_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
    },
    paths::path_relative_to_repo,
    state::CompletionState,
};
use std::path::Path;
pub(super) fn build_report(
    repo_root: &Path,
    absolute_output_path: &Path,
    checks: Vec<VcfAllRetainedToolsCompleteGoalCheck>,
    state: &CompletionState,
) -> VcfAllRetainedToolsCompleteReport {
    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    VcfAllRetainedToolsCompleteReport {
        schema_version: VCF_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        retained_row_count: state.retained_row,
        retained_stage_count: state.retained_stage,
        retained_tool_count: state.retained_tool,
        active_row_count: state.active_row,
        removed_row_count: state.removed_row,
        active_stage_count: state.active_stage,
        active_tool_count: state.active_tool,
        expected_result_row_count: state.expected_result_row,
        rendered_command_row_count: state.rendered_command_row,
        parser_fixture_row_count: state.parser_fixture_row,
        local_smoke_row_count: state.local_smoke_row,
        local_smoke_host_stage_row_count: state.local_smoke_host_stage_row,
        local_smoke_container_row_count: state.local_smoke_container_row,
        local_smoke_external_artifact_row_count: state.local_smoke_external_artifact_row,
        report_map_row_count: state.report_map_row,
        ok: failed_goal_count == 0
            && checks.len() == EXPECTED_CHECKED_GOAL_COUNT
            && state.retained_row == EXPECTED_RETAINED_ROW_COUNT
            && state.active_row == state.active_bindings.len()
            && state.expected_result_row == state.active_row
            && state.rendered_command_row == state.active_row
            && state.parser_fixture_row == state.active_row
            && state.report_map_row == state.active_row
            && state.local_smoke_row == EXPECTED_RETAINED_ROW_COUNT,
        checks,
    }
}
