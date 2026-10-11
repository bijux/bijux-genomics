use super::{
    checks, model::VcfAdaptersReadyReport, ports::VcfAdapterReadinessPorts,
    projection::build_vcf_adapters_ready_report, state::ReadinessState,
};
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
/// Publish all adapter-readiness checks over one admitted request observation set.
///
/// # Errors
/// Publishes the complete diagnostic before returning failed readiness; reports IO errors.
pub fn render_vcf_adapters_ready(
    run_id: &str,
    observations: &VcfAdapterObservations<'_>,
    ports: &VcfAdapterReadinessPorts,
    output_path: &Path,
) -> Result<VcfAdaptersReadyReport> {
    let repo_root = observations.repo_root();
    let absolute_output_path = repo_relative_path(repo_root, output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let mut checks = Vec::new();
    let mut state = ReadinessState::default();
    checks::registry_checks(observations, &mut state, &mut checks);
    checks::adapter_checks(observations, &mut state, &mut checks);
    checks::evidence_checks(run_id, observations, &mut state, &mut checks, ports);
    let report =
        build_vcf_adapters_ready_report(repo_root, &absolute_output_path, checks, state.counts);
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        bail!(
            "VCF adapter readiness gate failed for goals {}; inspect {}",
            report.failing_goal_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(", "),
            report.output_path
        );
    }
    Ok(report)
}
fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
