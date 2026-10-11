//! Semantic admission for persisted local HPC dry-run readiness reports.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcDryRunReadinessBehavior, BenchmarkLocalHpcDryRunReadinessReport,
    BenchmarkLocalHpcDryRunReadinessSummary, BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1,
};

const REQUIRED_GOAL_IDS: [u32; 9] = [481, 482, 483, 484, 485, 486, 487, 488, 489];

/// Validate goal closure, readiness behavior, summary counts, and paths.
///
/// # Errors
/// Returns an error when a persisted dry-run readiness report is inconsistent.
pub fn validate_benchmark_local_hpc_dry_run_readiness(
    report: &BenchmarkLocalHpcDryRunReadinessReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1 {
        return invalid("unsupported local HPC dry-run readiness schema");
    }
    require_path(&report.output_path, "output_path")?;
    if report.checks.len() != REQUIRED_GOAL_IDS.len()
        || report.checked_goal_count != report.checks.len()
        || report.checks.iter().map(|check| check.goal_id).collect::<Vec<_>>() != REQUIRED_GOAL_IDS
    {
        return invalid("local HPC dry-run readiness goal inventory drifted");
    }
    let mut goal_ids = BTreeSet::new();
    for check in &report.checks {
        if !goal_ids.insert(check.goal_id) {
            return invalid("local HPC dry-run readiness goal identity is duplicated");
        }
        require_text(&check.surface, "checks[].surface")?;
        require_text(&check.detail, "checks[].detail")?;
        if let Some(path) = &check.output_path {
            require_path(path, "checks[].output_path")?;
        }
    }
    let passed = report.checks.iter().filter(|check| check.ok).count();
    let failing = report
        .checks
        .iter()
        .filter_map(|check| (!check.ok).then_some(check.goal_id))
        .collect::<Vec<_>>();
    if report.passed_goal_count != passed
        || report.failed_goal_count != failing.len()
        || report.passed_goal_count + report.failed_goal_count != report.checked_goal_count
        || report.failing_goal_ids != failing
    {
        return invalid("local HPC dry-run readiness result counters drifted");
    }
    validate_behavior(&report.behavior, report.failed_goal_count == 0)?;
    if report.ready_for_first_hpc_run != (report.failed_goal_count == 0 && report.behavior.proven) {
        return invalid("local HPC dry-run readiness verdict drifted");
    }
    validate_summary(&report.summary, report.ready_for_first_hpc_run)
}

fn validate_behavior(
    behavior: &BenchmarkLocalHpcDryRunReadinessBehavior,
    all_goals_passed: bool,
) -> Result<(), BenchError> {
    if behavior.goals_481_to_489_validated != all_goals_passed {
        return invalid("local HPC dry-run readiness goal behavior drifted");
    }
    let no_ambiguity = behavior.goals_481_to_489_validated
        && behavior.benchmark_jobs_keep_known_assets
        && behavior.benchmark_jobs_keep_scratch_layout
        && behavior.benchmark_jobs_keep_explicit_execution_resolution
        && behavior.pipeline_nodes_keep_dependency_manifests
        && behavior.dependency_failures_block_descendants_only
        && behavior.resume_rules_are_explicit
        && behavior.result_collection_statuses_are_distinct
        && behavior.first_run_candidate_is_small_and_governed;
    if behavior.no_manual_steps_or_ambiguity != no_ambiguity
        || behavior.proven != behavior.no_manual_steps_or_ambiguity
    {
        return invalid("local HPC dry-run readiness behavior closure drifted");
    }
    Ok(())
}

fn validate_summary(
    summary: &BenchmarkLocalHpcDryRunReadinessSummary,
    ready: bool,
) -> Result<(), BenchError> {
    if summary.stage_benchmark_row_count > summary.benchmark_job_count
        || summary.pipeline_node_row_count > summary.essential_pipeline_job_count
        || summary.candidate_job_count > summary.benchmark_job_count
        || summary.candidate_representative_group_count > summary.candidate_job_count
    {
        return invalid("local HPC dry-run readiness summary cardinality drifted");
    }
    if ready
        && (summary.benchmark_job_count == 0
            || summary.essential_pipeline_job_count == 0
            || summary.staged_input_count == 0
            || summary.execution_resolver_row_count == 0
            || summary.execution_unclassified_tool_count != 0
            || summary.stage_benchmark_row_count != summary.benchmark_job_count
            || summary.pipeline_node_row_count != summary.essential_pipeline_job_count
            || summary.dependency_case_count == 0
            || summary.resume_rerun_job_count == 0
            || summary.result_collection_row_count == 0
            || summary.candidate_job_count == 0
            || summary.candidate_representative_group_count == 0
            || summary.candidate_selected_input_bytes == 0)
    {
        return invalid("ready local HPC dry-run summary is incomplete or ambiguous");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC dry-run readiness requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local HPC dry-run readiness requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
