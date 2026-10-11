use super::{service::collect_hpc_candidate_run_manifest, CandidateContext};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::BenchmarkLocalHpcCandidateRunManifest as LocalHpcCandidateRunManifest;
use std::{
    fs,
    path::{Path, PathBuf},
};
/// Publish the canonical locally selected candidate manifest.
///
/// # Errors
/// Returns input admission, selection, output IO or replay drift failures.
pub fn render_hpc_candidate_run_manifest(
    context: &CandidateContext<'_>,
    output_path: PathBuf,
) -> Result<LocalHpcCandidateRunManifest> {
    let repo_root = context.repo_root;
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = collect_hpc_candidate_run_manifest(context, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)?;
    Ok(report)
}
/// Compare the retained manifest with a freshly selected candidate.
///
/// # Errors
/// Returns input admission, selection, output IO or replay drift failures.
pub fn validate_hpc_candidate_run_manifest_path(
    context: &CandidateContext<'_>,
    manifest_path: &Path,
) -> Result<LocalHpcCandidateRunManifest> {
    let repo_root = context.repo_root;
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = collect_hpc_candidate_run_manifest(context, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected =
        serde_json::to_vec_pretty(&report).context("serialize governed HPC candidate manifest")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC candidate run manifest `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-candidate-run-manifest --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    Ok(report)
}
