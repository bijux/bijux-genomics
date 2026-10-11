use super::{
    models::BamActiveRowConsistencyReport, service::build_bam_active_row_consistency_report,
};
use crate::{publication::repo_relative_path, EvidenceContext};
use anyhow::{anyhow, Result};
use std::path::Path;
/// Publish all six consistency diagnostics before reporting a failed assessment.
///
/// # Errors
/// Returns source failures, publication failures, or inconsistent active bindings.
pub fn render_bam_active_row_consistency(
    context: &EvidenceContext<'_>,
    output_path: &Path,
) -> Result<BamActiveRowConsistencyReport> {
    let output_path = repo_relative_path(context.repo_root, output_path);
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }

    let report = build_bam_active_row_consistency_report(context, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam active rows must stay count-consistent across rendered commands, output declarations, expected results, parser fixtures, local jobs, and report-map rows"
        ));
    }
    Ok(report)
}
