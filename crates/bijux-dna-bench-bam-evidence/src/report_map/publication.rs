use super::{
    declarations::BAM_REPORT_MAP_SCHEMA_VERSION, models::BamReportMapReport,
    service::collect_report, tsv::render_bam_report_map_tsv,
};
use crate::{
    publication::{path_relative_to_repo, repo_relative_path},
    EvidenceContext,
};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn render_bam_report_map(
    context: &EvidenceContext<'_>,
    output_path: &Path,
) -> Result<BamReportMapReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let (expected_result_row_count, rows) = collect_report(context)?;
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let section_count =
        rows.iter().map(|row| row.report_section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut section_counts = BTreeMap::<String, usize>::new();
    let mut branch_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *section_counts.entry(row.report_section_id.clone()).or_default() += 1;
        *branch_counts
            .entry(row.workflow_branch_id.clone().unwrap_or_else(|| "core_alignment".to_string()))
            .or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, render_bam_report_map_tsv(&rows).as_bytes())?;

    Ok(BamReportMapReport {
        schema_version: BAM_REPORT_MAP_SCHEMA_VERSION,
        domain: "bam",
        output_path: path_relative_to_repo(repo_root, &output_path),
        expected_result_row_count,
        row_count: rows.len(),
        stage_count,
        tool_count,
        section_count,
        summary_table_count,
        section_counts,
        branch_counts,
        rows,
    })
}
