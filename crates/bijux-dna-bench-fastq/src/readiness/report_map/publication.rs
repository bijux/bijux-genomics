use super::declarations::FASTQ_REPORT_MAP_SCHEMA_VERSION;
use super::format::render_fastq_report_map_tsv;
use super::models::FastqReportMapReport;
use super::paths::path_relative_to_repo;
use super::paths::repo_relative_path;
use super::projection::collect_fastq_report_map_rows;
use crate::readiness::declarations::ReportContext;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
/// Publish canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn render_fastq_report_map(
    context: &ReportContext<'_>,
    output_path: &Path,
) -> Result<FastqReportMapReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let expected_result_row_count = context
        .evidence
        .expected(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "fastq")
        .count();
    let rows = collect_fastq_report_map_rows(context)?;
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let section_count =
        rows.iter().map(|row| row.report_section_id.clone()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.clone()).collect::<BTreeSet<_>>().len();
    let mut section_counts = BTreeMap::<String, usize>::new();
    let mut stage_kind_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *section_counts.entry(row.report_section_id.clone()).or_default() += 1;
        *stage_kind_counts.entry(row.stage_kind.clone()).or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_fastq_report_map_tsv(&rows).as_bytes(),
    )
    .with_context(|| format!("write {}", output_path.display()))?;

    Ok(FastqReportMapReport {
        schema_version: FASTQ_REPORT_MAP_SCHEMA_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        expected_result_row_count,
        row_count: rows.len(),
        stage_count,
        tool_count,
        section_count,
        summary_table_count,
        section_counts,
        stage_kind_counts,
        rows,
    })
}
