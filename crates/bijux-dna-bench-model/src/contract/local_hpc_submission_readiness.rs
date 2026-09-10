//! Semantic admission for persisted local HPC submission-readiness reports.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcSubmissionReadinessReport,
    BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1,
};

const REQUIRED_GOAL_COUNT: usize = 99;

/// Validate goal/category closure, result counters, verdict, and evidence paths.
///
/// # Errors
/// Returns an error when a persisted submission-readiness report is inconsistent.
pub fn validate_benchmark_local_hpc_submission_readiness(
    report: &BenchmarkLocalHpcSubmissionReadinessReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1 {
        return invalid("unsupported local HPC submission-readiness schema");
    }
    require_path(&report.output_path, "output_path")?;
    if report.checks.len() != REQUIRED_GOAL_COUNT
        || report.checked_goal_count != report.checks.len()
    {
        return invalid("local HPC submission-readiness goal count drifted");
    }

    let mut goal_ids = BTreeSet::new();
    for check in &report.checks {
        if !goal_ids.insert(check.goal_id) {
            return invalid("local HPC submission-readiness goal identity is duplicated");
        }
        if check.category != required_category(check.goal_id)? {
            return invalid("local HPC submission-readiness goal category drifted");
        }
        require_text(&check.surface, "checks[].surface")?;
        require_text(&check.detail, "checks[].detail")?;
        if let Some(path) = &check.output_path {
            require_path(path, "checks[].output_path")?;
        }
    }
    let required_goal_ids = (1..=99).collect::<BTreeSet<_>>();
    if goal_ids != required_goal_ids {
        return invalid("local HPC submission-readiness goal inventory drifted");
    }

    let passed = report.checks.iter().filter(|check| check.ok).count();
    let failing = report
        .checks
        .iter()
        .filter_map(|check| (!check.ok).then_some(check.goal_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if report.passed_goal_count != passed
        || report.failed_goal_count != failing.len()
        || report.passed_goal_count + report.failed_goal_count != report.checked_goal_count
        || report.failing_goal_ids != failing
        || report.ok != report.failing_goal_ids.is_empty()
    {
        return invalid("local HPC submission-readiness result closure drifted");
    }
    Ok(())
}

fn required_category(goal_id: u32) -> Result<&'static str, BenchError> {
    match goal_id {
        1 | 28 => Ok("stage_matrices"),
        2..=27 | 29..=52 => Ok("local_smokes"),
        53..=64 => Ok("benchmark_harness"),
        65..=76 => Ok("corpora"),
        77..=90 => Ok("pipeline_dags"),
        91..=99 => Ok("slurm_dry_run"),
        _ => invalid("local HPC submission-readiness goal is outside the governed inventory"),
    }
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC submission-readiness requires non-empty {field}"));
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
            "local HPC submission-readiness requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}

mod slurm;
pub use slurm::{evaluate_benchmark_local_hpc_slurm_readiness, BenchmarkLocalHpcSlurmObservation};
