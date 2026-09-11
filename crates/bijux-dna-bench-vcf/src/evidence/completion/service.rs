use super::{
    model::VcfAllRetainedToolsCompleteReport, paths::repo_relative_path, ports::VcfCompletionPorts,
    state::CompletionState,
};
use anyhow::{anyhow, Context, Result};
use std::path::Path;
/// Execute every governed retained-tool check and publish the complete diagnostic.
///
/// # Errors
/// Returns failed readiness after publishing all check results; publication failures propagate.
pub fn render_vcf_all_retained_tools_complete(
    repo_root: &Path,
    output_path: &Path,
    ports: &VcfCompletionPorts<'_>,
) -> Result<VcfAllRetainedToolsCompleteReport> {
    let absolute_output_path = repo_relative_path(repo_root, output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let mut checks = Vec::new();
    let mut state = CompletionState::default();
    super::checks::collect(repo_root, ports, &mut state, &mut checks);
    let report = super::projection::build_report(repo_root, &absolute_output_path, checks, &state);
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "every governed VCF retained-tool gate from goals 336 through 359 must remain complete"
        ));
    }
    Ok(report)
}
