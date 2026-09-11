use super::{
    collect_expected_benchmark_result_rows, tsv::render_expected_benchmark_results_tsv,
    ExpectedBenchmarkResultsReport, EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
};
use crate::{
    publication::{path_relative_to_repo, repo_relative_path},
    ExpectedContext,
};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
/// Project admitted canonical declarations into result evidence.
///
/// # Errors
/// Returns acquisition, binding admission or publication failure.
pub fn render_expected_benchmark_results(
    context: &ExpectedContext<'_>,
    output_path: &Path,
) -> Result<ExpectedBenchmarkResultsReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_expected_benchmark_result_rows(context)?;
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut fixture_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *fixture_counts.entry(row.fixture_id.clone()).or_default() += 1;
    }

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_expected_benchmark_results_tsv(&rows).as_bytes(),
    )?;

    Ok(ExpectedBenchmarkResultsReport {
        schema_version: EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        domain_counts,
        fixture_counts,
        rows,
    })
}
