use super::{
    acquisition::EvidenceSet,
    declarations::{
        BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT, EXPECTED_BAM_COMPARABLE_STAGE_COUNT,
        EXPECTED_CHECKED_GOAL_COUNT, EXPECTED_MICRO_SMOKE_FAMILY_COUNT,
    },
    inventory::BindingInventory,
    models::{BamAllRetainedToolsCompleteGoalCheck, BamAllRetainedToolsCompleteReport},
    paths::path_relative_to_repo,
};
use std::path::Path;
pub(super) fn project_report(
    repo_root: &Path,
    absolute_output_path: &Path,
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
    checks: Vec<BamAllRetainedToolsCompleteGoalCheck>,
) -> BamAllRetainedToolsCompleteReport {
    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    BamAllRetainedToolsCompleteReport {
        schema_version: BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        retained_row_count: evidence.tool_map.row_count,
        retained_stage_count: evidence.tool_map.stage_count,
        retained_tool_count: evidence.tool_map.tool_count,
        command_adapter_row_count: evidence.command_coverage.row_count,
        output_declaration_row_count: evidence.output_declarations.row_count,
        expected_result_row_count: bindings.expected_result_bindings.len(),
        rendered_command_row_count: evidence.rendered_commands.row_count,
        parser_fixture_row_count: evidence.parser_coverage.row_count,
        local_smoke_row_count: evidence.local_smoke.row_count,
        local_smoke_host_stage_row_count: evidence.local_smoke.host_stage_smoke_row_count,
        local_smoke_container_row_count: evidence.local_smoke.container_smoke_row_count,
        local_smoke_external_artifact_row_count: evidence.local_smoke.external_artifact_row_count,
        report_map_row_count: evidence.report_map.row_count,
        active_row_consistency_surface_count: evidence.active_row_consistency.checked_surface_count,
        micro_smoke_family_count: evidence.micro_smoke_summary.family_count,
        science_threshold_stage_count: evidence.science_thresholds.stage_row_count,
        ok: failed_goal_count == 0
            && checks.len() == EXPECTED_CHECKED_GOAL_COUNT
            && bindings.expected_retained_row_count > 0
            && evidence.tool_map.row_count == bindings.expected_retained_row_count
            && evidence.command_coverage.row_count == bindings.expected_retained_row_count
            && evidence.output_declarations.row_count == bindings.expected_retained_row_count
            && bindings.expected_result_bindings.len() == bindings.expected_retained_row_count
            && evidence.rendered_commands.row_count == bindings.expected_retained_row_count
            && evidence.parser_coverage.row_count == bindings.expected_retained_row_count
            && evidence.local_smoke.row_count == bindings.expected_retained_row_count
            && evidence.report_map.row_count == bindings.expected_retained_row_count
            && evidence.active_row_consistency.checked_surface_count
                == EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT
            && evidence.micro_smoke_summary.family_count == EXPECTED_MICRO_SMOKE_FAMILY_COUNT
            && evidence.micro_smoke_summary.passes_behavior_test
            && evidence.science_thresholds.stage_row_count == EXPECTED_BAM_COMPARABLE_STAGE_COUNT
            && evidence.science_thresholds.missing_threshold_stage_count == 0
            && evidence.science_thresholds.missing_threshold_metric_count == 0,
        checks,
    }
}
