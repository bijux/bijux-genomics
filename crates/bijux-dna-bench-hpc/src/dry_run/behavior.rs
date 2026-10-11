use super::evidence::DryRunEvidence;
use bijux_dna_bench_model::{
    BenchmarkLocalHpcDryRunReadinessBehavior as LocalHpcDryRunReadyBehavior,
    BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck,
};
pub(in crate::dry_run) fn behavior(
    evidence: &DryRunEvidence,
    checks: &[LocalHpcDryRunReadyGoalCheck],
) -> LocalHpcDryRunReadyBehavior {
    let scratch_report = evidence.scratch.as_ref().ok();
    let execution_report = evidence.execution.as_ref().ok();
    let pipeline_node_report = evidence.pipeline_node.as_ref().ok();
    let dependency_report = evidence.dependency.as_ref().ok();
    let resume_report = evidence.resume.as_ref().ok();
    let result_collection_report = evidence.result_collection.as_ref().ok();
    let candidate_report = evidence.candidate.as_ref().ok();

    let goals_481_to_489_validated = checks.iter().all(|check| check.ok);
    let benchmark_jobs_keep_known_assets =
        candidate_report.is_some_and(|report| report.behavior.only_known_assets);
    let benchmark_jobs_keep_scratch_layout = scratch_report
        .is_some_and(|report| report.benchmark_job_count > 0 && report.input_link_count > 0);
    let benchmark_jobs_keep_explicit_execution_resolution =
        candidate_report.is_some_and(|report| {
            report.behavior.only_known_execution_modes
                && report.behavior.only_available_execution_resolution
        }) && execution_report.is_some_and(|report| report.row_count > 0);
    let pipeline_nodes_keep_dependency_manifests = pipeline_node_report
        .is_some_and(|report| report.pipeline_job_count > 0 && report.dependency_count > 0);
    let dependency_failures_block_descendants_only =
        dependency_report.is_some_and(|report| report.all_cases_proven);
    let resume_rules_are_explicit = resume_report.is_some_and(|report| report.behavior.proven);
    let result_collection_statuses_are_distinct =
        result_collection_report.is_some_and(|report| report.behavior.proven);
    let first_run_candidate_is_small_and_governed = candidate_report
        .is_some_and(|report| report.behavior.proven && report.selected_job_count > 0);
    let no_manual_steps_or_ambiguity = goals_481_to_489_validated
        && benchmark_jobs_keep_known_assets
        && benchmark_jobs_keep_scratch_layout
        && benchmark_jobs_keep_explicit_execution_resolution
        && pipeline_nodes_keep_dependency_manifests
        && dependency_failures_block_descendants_only
        && resume_rules_are_explicit
        && result_collection_statuses_are_distinct
        && first_run_candidate_is_small_and_governed;

    LocalHpcDryRunReadyBehavior {
        goals_481_to_489_validated,
        benchmark_jobs_keep_known_assets,
        benchmark_jobs_keep_scratch_layout,
        benchmark_jobs_keep_explicit_execution_resolution,
        pipeline_nodes_keep_dependency_manifests,
        dependency_failures_block_descendants_only,
        resume_rules_are_explicit,
        result_collection_statuses_are_distinct,
        first_run_candidate_is_small_and_governed,
        no_manual_steps_or_ambiguity,
        proven: no_manual_steps_or_ambiguity,
    }
}
