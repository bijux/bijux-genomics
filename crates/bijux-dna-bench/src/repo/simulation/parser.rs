//! Owner: bounded acquisition of typed illustrative parser evidence.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    project_all_domain_simulation_parser_row, AllDomainFakeRunMetrics,
    AllDomainFakeRunResultReport, AllDomainParserCollectorRow, BenchmarkStageResultManifest,
};
use serde::de::DeserializeOwned;
use std::path::Path;

/// Load metrics and a stage manifest and admit their binding to one simulation result.
///
/// # Errors
/// Rejects unreadable, oversized, malformed or contradictory evidence.
pub fn load_all_domain_simulation_parser_row(
    repo_root: &Path,
    result: &AllDomainFakeRunResultReport,
) -> Result<AllDomainParserCollectorRow> {
    let metrics: AllDomainFakeRunMetrics = read_document(&repo_root.join(&result.metrics_path))?;
    let manifest: BenchmarkStageResultManifest =
        read_document(&repo_root.join(&result.stage_result_path))?;
    project_all_domain_simulation_parser_row(result, &metrics, &manifest)
        .with_context(|| format!("admit simulation parser evidence for {}", result.result_id))
}

fn read_document<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
        .with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}
