//! Pure admission of micro-benchmark reporting observations and summary closure.
use crate::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkReport, MicroBenchmarkResultRow,
    MicroBenchmarkRunManifest, MICRO_BENCHMARK_REPORT_SCHEMA_VERSION,
};
use anyhow::{bail, Result};

/// Qualify report closure against its source run and observed result rows.
/// A passing report contract does not independently certify scientific execution.
///
/// # Errors
/// Rejects unsupported versions, contradictory counts, missing explanations or source disagreement.
pub fn validate_micro_benchmark_report(
    manifest: &MicroBenchmarkRunManifest,
    result_rows: &[MicroBenchmarkResultRow],
    report: &mut MicroBenchmarkReport,
) -> Result<()> {
    report.passes_behavior_test = false;
    validate_report_row_counts(report)?;
    validate_report_source_rows(result_rows, report)?;

    if report.micro_run_manifest_path != manifest.manifest_path {
        bail!("micro benchmark report manifest path drifted from the governed run manifest");
    }
    if report.result_row_count != result_rows.len()
        || manifest.result_row_count != result_rows.len()
    {
        bail!("micro benchmark report result_row_count drifted from the governed micro run");
    }
    if report.complete_row_count.checked_add(report.unavailable_row_count)
        != Some(report.result_row_count)
    {
        bail!(
            "micro benchmark report must account for every result row as complete or unavailable"
        );
    }
    if report.runtime_row_count != report.result_row_count {
        bail!("micro benchmark report runtime rows must cover every result row");
    }
    if report.memory_source_row_count != report.result_row_count {
        bail!("micro benchmark report memory-source rows must cover every result row");
    }
    if report.science_threshold_row_count == 0 {
        bail!("micro benchmark report must retain at least one science-threshold row");
    }
    if report.unavailable_row_count
        != result_rows
            .iter()
            .filter(|row| row.status != MicroBenchmarkExecutionStatus::Succeeded)
            .count()
    {
        bail!("micro benchmark report unavailable rows drifted from the governed run status slice");
    }
    let execution_status_total = report
        .execution_status_counts
        .values()
        .try_fold(0_usize, |total, count| total.checked_add(*count));
    if execution_status_total != Some(report.result_row_count) {
        bail!("micro benchmark report execution-status counts must sum to result_row_count");
    }
    if report.failed_rows.iter().any(|row| row.detail.trim().is_empty())
        || report.missing_rows.iter().any(|row| row.detail.trim().is_empty())
        || report.unavailable_rows.iter().any(|row| row.reason.trim().is_empty())
        || report.insufficient_data_rows.iter().any(|row| row.detail.trim().is_empty())
    {
        bail!("micro benchmark report rows must keep stable detail and reason fields");
    }
    report.passes_behavior_test = true;
    Ok(())
}

fn validate_report_row_counts(report: &MicroBenchmarkReport) -> Result<()> {
    if report.schema_version != MICRO_BENCHMARK_REPORT_SCHEMA_VERSION {
        bail!("unsupported micro benchmark report schema version");
    }
    for (declared, observed, label) in [
        (report.complete_row_count, report.complete_rows.len(), "complete"),
        (report.failed_row_count, report.failed_rows.len(), "failed"),
        (report.missing_row_count, report.missing_rows.len(), "missing"),
        (report.unavailable_row_count, report.unavailable_rows.len(), "unavailable"),
        (
            report.insufficient_data_row_count,
            report.insufficient_data_rows.len(),
            "insufficient data",
        ),
        (report.runtime_row_count, report.runtime_rows.len(), "runtime"),
        (report.memory_source_row_count, report.memory_source_rows.len(), "memory source"),
        (
            report.science_threshold_row_count,
            report.science_threshold_rows.len(),
            "science threshold",
        ),
    ] {
        if declared != observed {
            bail!("micro benchmark report {label} row count disagrees with its rows");
        }
    }
    Ok(())
}

fn counted<T: Ord>(values: impl IntoIterator<Item = T>) -> std::collections::BTreeMap<T, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
}

fn validate_report_source_rows(
    results: &[MicroBenchmarkResultRow],
    report: &MicroBenchmarkReport,
) -> Result<()> {
    let expected = counted(results.iter().map(|r| {
        (
            r.component_id.as_str(),
            r.execution_id.as_str(),
            r.domain.as_str(),
            r.stage_id.as_str(),
            r.tool_id.as_str(),
            r.status.as_str(),
        )
    }));
    let complete = report.complete_rows.iter().map(|r| {
        (
            r.component_id.as_str(),
            r.execution_id.as_str(),
            r.domain.as_str(),
            r.stage_id.as_str(),
            r.tool_id.as_str(),
            MicroBenchmarkExecutionStatus::Succeeded.as_str(),
        )
    });
    let unavailable = report.unavailable_rows.iter().map(|r| {
        (
            r.component_id.as_str(),
            r.execution_id.as_str(),
            r.domain.as_str(),
            r.stage_id.as_str(),
            r.tool_id.as_str(),
            r.execution_status.as_str(),
        )
    });
    if counted(complete.chain(unavailable)) != expected {
        bail!("micro benchmark report result identities or statuses disagree with source results");
    }
    let runtime = counted(report.runtime_rows.iter().map(|r| {
        (
            r.component_id.as_str(),
            r.execution_id.as_str(),
            r.domain.as_str(),
            r.stage_id.as_str(),
            r.tool_id.as_str(),
            r.execution_status.as_str(),
        )
    }));
    if runtime != expected {
        bail!("micro benchmark runtime rows disagree with source identities or statuses");
    }
    let memory = counted(report.memory_source_rows.iter().map(|r| {
        (
            r.component_id.as_str(),
            r.execution_id.as_str(),
            r.domain.as_str(),
            r.stage_id.as_str(),
            r.tool_id.as_str(),
            r.execution_status.as_str(),
        )
    }));
    if memory != expected {
        bail!("micro benchmark memory rows disagree with source identities or statuses");
    }
    if counted(results.iter().map(|r| r.status.as_str().to_string()))
        != report.execution_status_counts
    {
        bail!("micro benchmark execution-status counts disagree with source results");
    }
    if counted(report.runtime_rows.iter().map(|r| r.runtime_source.clone()))
        != report.runtime_source_counts
    {
        bail!("micro benchmark runtime-source counts disagree with observed rows");
    }
    if counted(report.memory_source_rows.iter().map(|r| r.memory_source.clone()))
        != report.memory_source_counts
    {
        bail!("micro benchmark memory-source counts disagree with observed rows");
    }
    Ok(())
}
