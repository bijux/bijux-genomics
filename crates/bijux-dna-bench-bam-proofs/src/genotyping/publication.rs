use super::models::BamGenotypingCompleteReport;
#[cfg(feature = "bam_downstream")]
use super::service::build_bam_genotyping_complete_report;
use crate::downstream::DownstreamContext;
use anyhow::anyhow;
#[cfg(feature = "bam_downstream")]
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
/// Assess and publish downstream BAM scientific completion.
///
/// # Errors
/// Returns feature, acquisition, scientific admission or publication failure.
#[cfg(feature = "bam_downstream")]
pub fn render_bam_genotyping_complete(
    context: &DownstreamContext<'_>,
    output_path: &Path,
) -> Result<BamGenotypingCompleteReport> {
    let repo_root = context.proofs.repo_root;
    // Publication resolves against the request root; report paths remain repository-relative.
    let output_path = repo_root.join(output_path);
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_genotyping_complete_report(context, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam genotyping completion must keep active scope, command, output, parser, expected-result, report, schema, local plan, exact vcf handoff, and downstream variant-count plus missingness proof"
        ));
    }
    Ok(report)
}

/// Report the unavailable downstream feature before any evidence effects.
///
/// # Errors
/// Always returns the retained feature requirement.
#[cfg(not(feature = "bam_downstream"))]
pub fn unavailable() -> Result<BamGenotypingCompleteReport> {
    Err(anyhow!("bam.genotyping completion requires the `bam_downstream` feature"))
}

/// Assess and publish downstream BAM scientific completion.
///
/// # Errors
/// Returns feature, acquisition, scientific admission or publication failure.
#[cfg(not(feature = "bam_downstream"))]
pub fn render_bam_genotyping_complete(
    _context: &DownstreamContext<'_>,
    _output_path: &Path,
) -> Result<BamGenotypingCompleteReport> {
    unavailable()
}
