use super::models::BamCommandAdapterCoverageRow;
use super::projection::render_coverage_row;
use crate::EvidenceContext;
use anyhow::Result;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn collect_bam_command_adapter_coverage_rows(
    context: &EvidenceContext<'_>,
) -> Result<(usize, usize, Vec<BamCommandAdapterCoverageRow>)> {
    let repo_root = context.repo_root;
    let serving = context.serving.serving(repo_root)?;
    let stage_count = serving.stage_count;
    let tool_count = serving.tool_count;
    let serving_rows = serving.rows;
    let rows = serving_rows.iter().map(render_coverage_row).collect::<Vec<_>>();
    Ok((stage_count, tool_count, rows))
}
