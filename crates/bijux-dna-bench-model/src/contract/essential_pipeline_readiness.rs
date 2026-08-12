//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted essential-pipeline readiness evidence.

use crate::{
    model::{
        BenchmarkEssentialPipelineReadinessReport, BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS,
        BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH,
    },
    BenchError,
};

use super::BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1;

const PIPELINE_COUNT: usize = 10;
const NODE_COUNT: usize = 93;
const OUTPUT_COUNT: usize = 267;

/// Validate the complete persisted essential-pipeline readiness record.
///
/// Failure records are valid evidence when their goal identity and aggregates close.
///
/// # Errors
/// Returns an error when goal identity, counts, or aggregate state drift.
pub fn validate_benchmark_essential_pipeline_readiness(
    report: &BenchmarkEssentialPipelineReadinessReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1 {
        return invalid("unsupported essential-pipeline readiness schema");
    }
    if report.output_path != BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH {
        return invalid("essential-pipeline readiness output path is not canonical");
    }
    if report.checks.len() != BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS.len() {
        return invalid("essential-pipeline readiness goal roster is incomplete");
    }
    for (check, goal) in report.checks.iter().zip(BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS) {
        let expected_surface = if goal.goal_id <= 270 {
            format!("pipeline {}", goal.surface)
        } else {
            goal.surface.to_string()
        };
        if check.goal_id != goal.goal_id
            || check.surface != expected_surface
            || check.output_path.as_deref() != Some(goal.output_path)
        {
            return invalid(
                "essential-pipeline readiness goal identity or output contract drifted",
            );
        }
        require_text(&check.detail, "checks[].detail")?;
    }

    validate_aggregates(report)?;
    validate_success_counts(report)?;
    Ok(())
}

fn validate_aggregates(
    report: &BenchmarkEssentialPipelineReadinessReport,
) -> Result<(), BenchError> {
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
        return invalid("essential-pipeline readiness aggregates do not close");
    }
    Ok(())
}

fn validate_success_counts(
    report: &BenchmarkEssentialPipelineReadinessReport,
) -> Result<(), BenchError> {
    if report.pipeline_count != PIPELINE_COUNT {
        return invalid("essential-pipeline readiness pipeline count drifted");
    }
    let goal_passed =
        |goal_id| report.checks.iter().any(|check| check.goal_id == goal_id && check.ok);
    if report.checks[..PIPELINE_COUNT].iter().all(|check| check.ok)
        && report.dag_node_count != NODE_COUNT
    {
        return invalid("essential-pipeline readiness DAG-node count drifted");
    }
    for (goal_id, count, expected, label) in [
        (271, report.corpus_asset_row_count, NODE_COUNT, "corpus/assets"),
        (272, report.rendered_command_row_count, NODE_COUNT, "rendered-command"),
        (273, report.fake_run_node_count, NODE_COUNT, "fake-run node"),
        (273, report.fake_run_output_count, OUTPUT_COUNT, "fake-run output"),
        (276, report.report_map_row_count, OUTPUT_COUNT, "report-map row"),
    ] {
        if goal_passed(goal_id) && count != expected {
            return invalid(format!("essential-pipeline readiness {label} count drifted"));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("essential-pipeline readiness requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
