//! Owner: benchmark dependency-source inspection and admitted report publication.
use crate::repo::command_artifacts::slurm_dependencies;
use anyhow::{ensure, Result};
use bijux_dna_bench_model::{
    build_benchmark_local_slurm_dependency_report, inspect_benchmark_local_slurm_job_dependencies,
    BenchmarkLocalSlurmDependencyCheckReport,
};
use std::path::{Path, PathBuf};

/// Inspect current script dependency declarations and publish the completed diagnostic.
///
/// # Errors
/// Rejects unsafe locations, unreadable inputs, malformed evidence or conflicting sources.
/// Completed negative diagnostics are published before returning their findings error.
pub fn inspect_benchmark_local_slurm_dependencies(
    repo_root: &Path,
    root_path: PathBuf,
    manifest_path: PathBuf,
    report_path: PathBuf,
) -> Result<BenchmarkLocalSlurmDependencyCheckReport> {
    let paths = slurm_dependencies::prepare(repo_root, &root_path, &manifest_path, &report_path)?;
    let manifest = slurm_dependencies::read_manifest(&paths)?;
    let mut jobs = Vec::with_capacity(manifest.jobs.len());
    for job in &manifest.jobs {
        let (path, body) = slurm_dependencies::read_script(&paths, &job.script_path)?;
        jobs.push(inspect_benchmark_local_slurm_job_dependencies(job, path, &body));
    }
    let report = build_benchmark_local_slurm_dependency_report(
        paths.scripts.display(&paths.scripts.root),
        paths.scripts.display(&paths.manifest),
        paths.scripts.display(&paths.scripts.report),
        &manifest,
        jobs,
    )?;
    slurm_dependencies::publish(&paths, &report)?;
    ensure!(report.ok, "slurm dependency validation failed; see {}", report.report_path);
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/readiness/slurm_dependencies.rs"]
mod tests;
