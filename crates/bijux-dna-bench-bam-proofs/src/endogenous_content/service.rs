use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_TOOL_ID;
use super::models::BamEndogenousContentCompleteReport;
use super::report::project;
use super::report::ReportProjection;
use super::rows::build_endogenous_content_complete_row;
use crate::ProofContext;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(super) fn build_bam_endogenous_content_complete_report(
    context: &ProofContext<'_>,
    output_path: &Path,
) -> Result<BamEndogenousContentCompleteReport> {
    let repo_root = context.repo_root;
    let overlap_endogenous_report = context.readiness.overlap_endogenous(repo_root)?;
    let endogenous_row = overlap_endogenous_report
        .rows
        .into_iter()
        .find(|row| row.stage_id == EXPECTED_STAGE_ID && row.tool_id == EXPECTED_TOOL_ID)
        .ok_or_else(|| {
            anyhow!(
                "bam overlap/endogenous readiness report is missing `{EXPECTED_STAGE_ID}` / `{EXPECTED_TOOL_ID}`"
            )
        })?;
    let row = build_endogenous_content_complete_row(repo_root, endogenous_row)?;
    let complete_row_count = usize::from(row.coverage_status == "complete");
    let rows = vec![row];
    let violations =
        rows.iter().filter(|row| row.coverage_status != "complete").cloned().collect::<Vec<_>>();

    Ok(project(ReportProjection { output_path, repo_root, complete_row_count, rows, violations }))
}
