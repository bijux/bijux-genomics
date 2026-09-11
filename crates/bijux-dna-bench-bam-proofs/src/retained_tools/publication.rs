use super::{
    models::BamAllRetainedToolsCompleteReport, paths::repo_relative_path, service::assess,
    RetainedContext,
};
use anyhow::{anyhow, Context, Result};
use std::path::Path;
/// Assess and publish retained BAM coverage from canonical provider evidence.
///
/// # Errors
/// Returns acquisition, publication or incomplete retained-coverage errors.
pub fn render_bam_all_retained_tools_complete(
    context: &RetainedContext<'_>,
    output_path: &Path,
) -> Result<BamAllRetainedToolsCompleteReport> {
    let repo_root = context.repo_root;

    let absolute_output_path = repo_relative_path(repo_root, output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let report = assess(context, &absolute_output_path)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "every governed BAM retained-tool gate from goals 431 through 449 must remain complete"
        ));
    }
    Ok(report)
}
