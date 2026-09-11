use super::VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION;
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::expectations::{
    VcfExpectedBenchmarkResultRow, VcfExpectedBenchmarkResultsReport,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};
/// Publish the compatible expected-result table after contract validation succeeds.
///
/// # Errors
/// Returns acquisition, projection, contract or atomic publication errors.
pub fn render_vcf_expected_benchmark_results(
    observations: &VcfAdapterObservations<'_>,
    output_path: &Path,
) -> Result<VcfExpectedBenchmarkResultsReport> {
    let repo_root = observations.repo_root();
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = super::collect_vcf_expected_benchmark_result_rows(observations)?;
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let corpus_count = rows.iter().map(|row| row.corpus_id.as_str()).collect::<BTreeSet<_>>().len();
    let asset_profile_count =
        rows.iter().map(|row| row.asset_profile_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
    }

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_vcf_expected_benchmark_results_tsv(&rows).as_bytes(),
    )?;

    Ok(VcfExpectedBenchmarkResultsReport {
        schema_version: VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count,
        tool_count,
        corpus_count,
        asset_profile_count,
        report_section_counts,
        rows,
    })
}

fn render_vcf_expected_benchmark_results_tsv(rows: &[VcfExpectedBenchmarkResultRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\texpected_outputs\texpected_metrics\treport_section\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.expected_outputs.join(",")),
            sanitize_tsv(&row.expected_metrics.join(",")),
            sanitize_tsv(&row.report_section),
        );
    }
    rendered
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
