use super::assessment::collect_fastq_adapter_output_contract_rows;
use super::assessment::row_has_adapter;
use super::declarations::FASTQ_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION;
use super::format::render_fastq_adapter_output_contract_tsv;
use super::models::FastqAdapterOutputContractReport;
use super::models::FastqAdapterOutputContractStatus;
use super::paths::path_relative_to_repo;
use super::paths::repo_relative_path;
use crate::readiness::declarations::OutputContext;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
/// Publish canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn render_fastq_adapter_output_contract(
    context: &OutputContext<'_>,
    output_path: &Path,
) -> Result<FastqAdapterOutputContractReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_fastq_adapter_output_contract_rows(context)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_fastq_adapter_output_contract_tsv(&rows).as_bytes(),
    )
    .with_context(|| format!("write {}", output_path.display()))?;

    let adapter_row_count = rows.iter().filter(|row| row_has_adapter(&row.adapter_status)).count();
    let complete_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row_has_adapter(&row.adapter_status)
                && row.output_contract_status == FastqAdapterOutputContractStatus::Complete
        })
        .count();
    let incomplete_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row_has_adapter(&row.adapter_status)
                && row.output_contract_status == FastqAdapterOutputContractStatus::Incomplete
        })
        .count();
    let missing_adapter_row_count = rows
        .iter()
        .filter(|row| {
            row.output_contract_status == FastqAdapterOutputContractStatus::MissingAdapter
        })
        .count();

    Ok(FastqAdapterOutputContractReport {
        schema_version: FASTQ_ADAPTER_OUTPUT_CONTRACT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        adapter_row_count,
        complete_adapter_row_count,
        incomplete_adapter_row_count,
        missing_adapter_row_count,
        rows,
    })
}
