use super::models::BamKinshipReadyReport;
use super::service::build_bam_kinship_ready_report;
use crate::readiness::paths::repo_relative_path;
use crate::readiness::ReadinessContext;
use anyhow::anyhow;
use anyhow::{Context, Result};
use std::path::Path;
/// Construct and publish the retained BAM readiness evidence.
///
/// # Errors
/// Returns acquisition, binding, proof or publication failure; failed assessments retain diagnostics.
pub fn render_bam_kinship_ready(
    context: &ReadinessContext<'_>,
    output_path: &Path,
) -> Result<BamKinshipReadyReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let report = build_bam_kinship_ready_report(context, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam kinship readiness must keep active scope, command, output, parser, expected-result, report, schema, and local proof"
        ));
    }
    Ok(report)
}
