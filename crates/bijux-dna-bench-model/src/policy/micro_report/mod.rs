//! Owner: pure micro report assembly over explicit component and repository observations.
mod coverage;
mod results;
use crate::{
    validate_micro_benchmark_report, MicroBenchmarkReport, MicroBenchmarkReportDestinations,
    MicroBenchmarkReportObservations, MicroBenchmarkReportSources,
    MICRO_BENCHMARK_REPORT_SCHEMA_VERSION,
};
use anyhow::Result;
use std::collections::BTreeMap;
/// Assemble and admit report coverage without loading files or publishing output.
/// # Errors
/// Rejects structural count and source contradictions; unavailable and failed observations remain explicit report rows.
pub fn summarize_micro_benchmark_report(
    sources: MicroBenchmarkReportSources<'_>,
    observations: MicroBenchmarkReportObservations,
    destinations: MicroBenchmarkReportDestinations,
) -> Result<MicroBenchmarkReport> {
    let manifest = sources.manifest;
    let result_rows = sources.results;
    let (complete_rows, unavailable_rows) = results::result_sections(result_rows);
    let (failed_rows, missing_rows) =
        coverage::coverage(sources, &observations.existing_evidence_paths);
    let insufficient_data_rows = observations.insufficient;
    let runtime_rows = observations.runtime;
    let memory_source_rows = observations.memory;
    let science_threshold_rows = observations.thresholds;
    let mut runtime_source_counts = BTreeMap::<String, usize>::new();
    let mut memory_source_counts = BTreeMap::<String, usize>::new();
    for row in &runtime_rows {
        *runtime_source_counts.entry(row.runtime_source.clone()).or_default() += 1;
    }
    for row in &memory_source_rows {
        *memory_source_counts.entry(row.memory_source.clone()).or_default() += 1;
    }
    let mut execution_status_counts = BTreeMap::<String, usize>::new();
    for row in result_rows {
        *execution_status_counts.entry(row.status.as_str().to_string()).or_default() += 1;
    }

    let mut report = MicroBenchmarkReport {
        schema_version: MICRO_BENCHMARK_REPORT_SCHEMA_VERSION.to_string(),
        markdown_output_path: destinations.markdown,
        json_output_path: destinations.json,
        micro_run_manifest_path: manifest.manifest_path.clone(),
        result_row_count: result_rows.len(),
        complete_row_count: complete_rows.len(),
        failed_row_count: failed_rows.len(),
        missing_row_count: missing_rows.len(),
        unavailable_row_count: unavailable_rows.len(),
        insufficient_data_row_count: insufficient_data_rows.len(),
        runtime_row_count: runtime_rows.len(),
        memory_source_row_count: memory_source_rows.len(),
        science_threshold_row_count: science_threshold_rows.len(),
        passes_behavior_test: false,
        execution_status_counts,
        runtime_source_counts,
        memory_source_counts,
        complete_rows,
        failed_rows,
        missing_rows,
        unavailable_rows,
        insufficient_data_rows,
        runtime_rows,
        memory_source_rows,
        science_threshold_rows,
    };
    validate_micro_benchmark_report(manifest, result_rows, &mut report)?;

    Ok(report)
}
