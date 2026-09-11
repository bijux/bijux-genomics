use super::{
    model::{VcfParsersReportReadyGoalCheck, VcfParsersReportReadyReport},
    VCF_PARSERS_REPORT_READY_SCHEMA_VERSION,
};
use std::path::Path;
pub(super) fn build_vcf_parsers_report_ready_report(
    repo_root: &Path,
    output_path: &Path,
    checks: Vec<VcfParsersReportReadyGoalCheck>,
    counts: super::state::ReadinessCounts,
) -> VcfParsersReportReadyReport {
    let super::state::ReadinessCounts {
        parser_fixtures: parser_fixture_row_count,
        active_fixtures: active_parser_fixture_row_count,
        expected_results: expected_result_row_count,
        report_rows: report_map_row_count,
        metrics: comparable_metric_row_count,
    } = counts;
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);

    VcfParsersReportReadyReport {
        schema_version: VCF_PARSERS_REPORT_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        parser_fixture_row_count,
        active_parser_fixture_row_count,
        expected_result_row_count,
        report_map_row_count,
        comparable_metric_row_count,
        ok: failed_goal_count == 0,
        checks,
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
