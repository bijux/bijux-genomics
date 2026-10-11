use super::VCF_REPORT_MAP_SCHEMA_VERSION;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::reports::{VcfReportMapReport, VcfReportMapRow};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    path::{Path, PathBuf},
};
/// Publish the compatible report map after admitted expected-result projection.
///
/// # Errors
/// Returns expected-result acquisition, contract or atomic publication errors.
pub fn render_vcf_report_map(
    observations: &crate::adapters::observations::VcfAdapterObservations<'_>,
    output_path: &Path,
) -> Result<VcfReportMapReport> {
    let repo_root = observations.repo_root();
    let output_path = repo_relative_path(repo_root, output_path);
    let expected =
        crate::evidence::expectations::collect_vcf_expected_benchmark_result_rows(observations)?;
    let rows = super::collect_vcf_report_map_rows_from_expected(observations.inputs(), expected)?;
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let section_count =
        rows.iter().map(|row| row.section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table.as_str()).collect::<BTreeSet<_>>().len();
    let mut section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *section_counts.entry(row.section_id.clone()).or_default() += 1;
    }

    bijux_dna_infra::atomic_write_bytes(&output_path, render_vcf_report_map_tsv(&rows).as_bytes())?;

    Ok(VcfReportMapReport {
        schema_version: VCF_REPORT_MAP_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count,
        tool_count,
        section_count,
        summary_table_count,
        section_counts,
        rows,
    })
}
fn render_vcf_report_map_tsv(rows: &[VcfReportMapRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tsection_id\tsummary_table\tmetric_columns\tfailure_columns\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.section_id),
            sanitize_tsv(&row.summary_table),
            sanitize_tsv(&row.metric_columns.join(",")),
            sanitize_tsv(&row.failure_columns.join(",")),
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
