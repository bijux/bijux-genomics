//! Owner: micro-benchmark report observation, admission and publication use case.
use crate::repo::micro_report::{capture, publication::Destinations};
use anyhow::Result;
use bijux_dna_bench_model::{
    summarize_micro_benchmark_report, MicroBenchmarkReport, MicroBenchmarkReportSources,
};
use std::path::Path;
/// Component projections and report destinations for one local report request.
pub struct MicroBenchmarkReportRequest<'a> {
    pub repo_root: &'a Path,
    pub sources: MicroBenchmarkReportSources<'a>,
    pub markdown_output_path: &'a Path,
    pub json_output_path: &'a Path,
}
/// Capture local evidence, admit report coverage and publish its two representations.
/// # Errors
/// Returns acquisition, coverage, path-admission or publication failures.
pub fn generate_micro_benchmark_report(
    request: MicroBenchmarkReportRequest<'_>,
) -> Result<MicroBenchmarkReport> {
    let paths = Destinations::prepare(
        request.repo_root,
        request.markdown_output_path,
        request.json_output_path,
    )?;
    let observations = capture(request.repo_root, request.sources.results, paths.governed)?;
    let report = summarize_micro_benchmark_report(
        request.sources,
        observations,
        paths.display(request.repo_root),
    )?;
    paths.publish(&report)?;
    Ok(report)
}
