use super::models::{BamAdapterOutputContractReport, BamAdapterOutputContractStatus};
use super::projection::row_has_adapter;
use super::{
    declarations::BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
    service::collect_bam_adapter_output_contract_rows, tsv::render_bam_adapter_output_contract_tsv,
};
use crate::publication::{path_relative_to_repo, repo_relative_path};
use crate::EvidenceContext;
use anyhow::Result;
use std::path::Path;
/// Build the declared BAM evidence view from explicit canonical observations.
///
/// # Errors
/// Returns acquisition, contract, or publication failures with source diagnostics.
pub fn render_bam_adapter_output_contract(
    context: &EvidenceContext<'_>,
    output_path: &Path,
) -> Result<BamAdapterOutputContractReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_bam_adapter_output_contract_rows(context)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_bam_adapter_output_contract_tsv(&rows).as_bytes(),
    )?;

    let adapter_row_count = rows.iter().filter(|row| row_has_adapter(&row.adapter_status)).count();
    let complete_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row_has_adapter(&row.adapter_status)
                && row.output_contract_status == BamAdapterOutputContractStatus::Complete
        })
        .count();
    let incomplete_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row_has_adapter(&row.adapter_status)
                && row.output_contract_status == BamAdapterOutputContractStatus::Incomplete
        })
        .count();
    let missing_adapter_row_count = rows
        .iter()
        .filter(|row| row.output_contract_status == BamAdapterOutputContractStatus::MissingAdapter)
        .count();

    Ok(BamAdapterOutputContractReport {
        schema_version: BAM_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        adapter_row_count,
        complete_adapter_row_count,
        incomplete_adapter_row_count,
        missing_adapter_row_count,
        rows,
    })
}
