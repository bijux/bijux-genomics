//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted local VCF smoke-suite readiness.

use crate::{
    model::{
        BenchmarkLocalVcfSmokeSuiteReport, BENCHMARK_LOCAL_VCF_SMOKE_GOALS,
        BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH,
    },
    BenchError,
};

use super::BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1;

/// Validate the complete persisted local VCF smoke-suite readiness record.
///
/// # Errors
/// Returns an error when goal identity, output evidence, or aggregates drift.
pub fn validate_benchmark_local_vcf_smoke_suite(
    report: &BenchmarkLocalVcfSmokeSuiteReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1 {
        return invalid("unsupported local VCF smoke-suite schema");
    }
    if report.output_path != BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH {
        return invalid("local VCF smoke-suite output path is not canonical");
    }
    if report.checks.len() != BENCHMARK_LOCAL_VCF_SMOKE_GOALS.len() {
        return invalid("local VCF smoke-suite goal roster is incomplete");
    }
    for (check, goal) in report.checks.iter().zip(BENCHMARK_LOCAL_VCF_SMOKE_GOALS) {
        if check.goal_id != goal.goal_id
            || check.surface != goal.surface
            || check.output_path.as_deref() != Some(goal.output_path)
        {
            return invalid("local VCF smoke-suite goal identity or output contract drifted");
        }
        require_text(&check.detail, "checks[].detail")?;
    }

    let failing_goal_ids = report
        .checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.goal_id)
        .collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = report.checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);
    if report.checked_goal_count != checked_goal_count
        || report.passed_goal_count != passed_goal_count
        || report.failed_goal_count != failed_goal_count
        || report.failing_goal_ids != failing_goal_ids
        || report.ok != (failed_goal_count == 0)
    {
        return invalid("local VCF smoke-suite aggregates do not close");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local VCF smoke-suite requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
