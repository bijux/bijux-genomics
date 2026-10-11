use super::models::FastqValidateReadsReadyReport;
use super::paths::repo_relative_path;
use super::service::build_fastq_validate_reads_ready_report;
use crate::proofs::inputs::ValidationContext;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
/// Evaluate and publish admitted FASTQ stage proof.
///
/// # Errors
/// Preserves evidence, scientific acceptance and publication failures.
pub fn render_fastq_validate_reads_ready(
    context: &ValidationContext<'_>,
    output_path: &Path,
) -> Result<FastqValidateReadsReadyReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let report = build_fastq_validate_reads_ready_report(context, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "fastq.validate_reads retained validators must keep active scope, command, output, parser, expected-result, report, and normalization proof"
        ));
    }
    Ok(report)
}
