use super::{
    model::{VcfAdaptersReadyGoalCheck, VcfAdaptersReadyReport},
    VCF_ADAPTERS_READY_SCHEMA_VERSION,
};
use std::path::Path;
pub(super) fn build_vcf_adapters_ready_report(
    repo_root: &Path,
    output_path: &Path,
    checks: Vec<VcfAdaptersReadyGoalCheck>,
    counts: super::state::ReadinessCounts,
) -> VcfAdaptersReadyReport {
    let super::state::ReadinessCounts {
        benchmark_ready: benchmark_ready_pair_count,
        adapter_complete: adapter_complete_pair_count,
        output_complete: output_complete_pair_count,
        rendered_commands: rendered_command_pair_count,
    } = counts;
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);

    VcfAdaptersReadyReport {
        schema_version: VCF_ADAPTERS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        benchmark_ready_pair_count,
        adapter_complete_pair_count,
        output_complete_pair_count,
        rendered_command_pair_count,
        ok: failed_goal_count == 0,
        checks,
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
