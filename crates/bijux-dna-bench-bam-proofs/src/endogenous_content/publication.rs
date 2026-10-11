use super::models::BamEndogenousContentCompleteReport;
use super::paths::repo_relative_path;
use super::service::build_bam_endogenous_content_complete_report;
use crate::ProofContext;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
/// Assess and publish canonical BAM stage completion.
///
/// # Errors
/// Returns acquisition, malformed evidence, incomplete assessment or publication failure.
pub fn render_bam_endogenous_content_complete(
    context: &ProofContext<'_>,
    output_path: &Path,
) -> Result<BamEndogenousContentCompleteReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_endogenous_content_complete_report(context, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam.endogenous_content must keep active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, and normalized endogenous metrics complete"
        ));
    }
    Ok(report)
}
