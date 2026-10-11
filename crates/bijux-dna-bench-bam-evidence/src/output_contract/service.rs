use super::models::BamAdapterOutputContractRow;
use super::projection::render_output_contract_row;
use crate::EvidenceContext;
use anyhow::Result;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn collect_bam_adapter_output_contract_rows(
    context: &EvidenceContext<'_>,
) -> Result<Vec<BamAdapterOutputContractRow>> {
    let repo_root = context.repo_root;
    let tool_map = context.serving.publish_serving(repo_root)?;
    Ok(tool_map.rows.iter().map(|row| render_output_contract_row(context, row)).collect::<Vec<_>>())
}
