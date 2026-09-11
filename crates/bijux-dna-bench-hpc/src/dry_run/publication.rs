use super::{assessment::build_hpc_dry_run_ready, DryRunContext};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_hpc_dry_run_readiness,
    BenchmarkLocalHpcDryRunReadinessReport as LocalHpcDryRunReadyReport,
};
use std::fs;
use std::path::{Path, PathBuf};
/// Publish all dry-run diagnostics before returning an unready error.
///
/// # Errors
/// Returns admission, observation, serialization, publication or readiness failures.
pub fn render_hpc_dry_run_ready(
    context: &DryRunContext<'_>,
    output_path: PathBuf,
) -> Result<LocalHpcDryRunReadyReport> {
    let repo_root = context.repo_root;
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_dry_run_ready(context, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)
        .with_context(|| format!("write {}", absolute_output.display()))?;
    if report.ready_for_first_hpc_run {
        Ok(report)
    } else {
        Err(anyhow!("local HPC dry-run readiness failed; see {}", report.output_path))
    }
}
/// Validate the persisted schema and compare bytes with freshly acquired evidence.
///
/// # Errors
/// Returns admission, observation, serialization, publication or readiness failures.
pub fn validate_hpc_dry_run_ready_path(
    context: &DryRunContext<'_>,
    manifest_path: &Path,
) -> Result<LocalHpcDryRunReadyReport> {
    let repo_root = context.repo_root;
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    context.output.admit(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let admitted: LocalHpcDryRunReadyReport =
        serde_json::from_slice(&observed).with_context(|| {
            format!("parse HPC dry-run readiness report {}", absolute_manifest_path.display())
        })?;
    validate_benchmark_local_hpc_dry_run_readiness(&admitted).with_context(|| {
        format!(
            "HPC dry-run readiness report `{}` drifted from its declared observations",
            absolute_manifest_path.display()
        )
    })?;
    let report = build_hpc_dry_run_ready(context, &absolute_manifest_path)?;
    let expected =
        serde_json::to_vec_pretty(&report).context("serialize governed HPC dry-run readiness")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC dry-run readiness report `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-dry-run-ready --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    if report.ready_for_first_hpc_run {
        Ok(report)
    } else {
        Err(anyhow!("local HPC dry-run readiness failed; see {}", report.output_path))
    }
}
