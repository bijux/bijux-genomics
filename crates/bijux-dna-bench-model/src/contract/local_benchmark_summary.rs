//! Validation for persisted local benchmark summaries.

use std::collections::BTreeSet;

use crate::{
    derive_benchmark_local_readiness_status, BenchError, BenchmarkLocalReadinessStatus,
    BenchmarkLocalSummaryReport, BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
};

/// Validate one local benchmark summary.
///
/// # Errors
/// Returns an error when identity, cardinality, or readiness invariants are invalid.
pub fn validate_benchmark_local_summary(
    report: &BenchmarkLocalSummaryReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local benchmark-summary schema `{}`",
            report.schema_version
        ));
    }
    for (value, field) in [
        (&report.fake_run_root, "fake_run_root"),
        (&report.report_output_path, "report_output_path"),
        (&report.markdown_output_path, "markdown_output_path"),
        (&report.source_stage_command_manifest_path, "source_stage_command_manifest_path"),
        (&report.manifest_completion_report_path, "manifest_completion_report_path"),
        (&report.output_completion_report_path, "output_completion_report_path"),
        (&report.runtime_metrics_report_path, "runtime_metrics_report_path"),
        (&report.tool_comparison_template_path, "tool_comparison_template_path"),
    ] {
        require_text(value, field)?;
    }
    if report.stage_count == 0 || report.stage_count != report.stages.len() {
        return invalid("local benchmark-summary stage_count must equal a non-empty stages array");
    }

    let ready = report
        .stages
        .iter()
        .filter(|stage| stage.readiness_status == BenchmarkLocalReadinessStatus::Ready)
        .count();
    let incomplete = report.stage_count - ready;
    let failed = report
        .stages
        .iter()
        .filter(|stage| stage.readiness_status == BenchmarkLocalReadinessStatus::Failed)
        .count();
    if (report.ready_stage_count, report.incomplete_stage_count, report.failed_stage_count)
        != (ready, incomplete, failed)
    {
        return invalid("local benchmark-summary readiness counters do not match stage rows");
    }

    let mut stage_ids = BTreeSet::new();
    for stage in &report.stages {
        require_text(&stage.stage_id, "stages[].stage_id")?;
        require_text(&stage.tool_id, "stages[].tool_id")?;
        require_text(&stage.runtime_seconds, "stages[].runtime_seconds")?;
        require_text(&stage.memory_mb, "stages[].memory_mb")?;
        require_text(&stage.output_metric, "stages[].output_metric")?;
        if stage.declared_output_count
            != stage.present_output_count.saturating_add(stage.missing_output_count)
        {
            return invalid(format!(
                "local benchmark-summary stage `{}` has inconsistent output counters",
                stage.stage_id
            ));
        }
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("local benchmark-summary repeats stage `{}`", stage.stage_id));
        }
        let expected = derive_benchmark_local_readiness_status(
            stage.manifest_exists,
            stage.missing_output_count,
            stage.runtime_status,
        );
        if stage.readiness_status != expected {
            return invalid(format!(
                "local benchmark-summary stage `{}` readiness status does not match its evidence",
                stage.stage_id
            ));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local benchmark-summary requires non-empty {field}"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
