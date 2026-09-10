//! Owner: benchmark script inspection and failure-report publication.
use crate::repo::command_artifacts::slurm_scripts;
use anyhow::{ensure, Result};
use bijux_dna_bench_model::{
    build_benchmark_local_slurm_script_body_report, inspect_benchmark_local_slurm_script_body,
    BenchmarkLocalSlurmScriptBodyReport,
};
use std::path::{Path, PathBuf};

/// Inspect current scripts and publish admitted evidence, including failed inspections.
///
/// # Errors
/// Rejects invalid locations, unreadable scripts, or script-text findings. A completed
/// inspection publishes its report before returning a findings error.
pub fn validate_benchmark_local_slurm_script_bodies(
    repo_root: &Path,
    root_path: PathBuf,
    report_path: PathBuf,
) -> Result<BenchmarkLocalSlurmScriptBodyReport> {
    let paths = slurm_scripts::prepare(repo_root, &root_path, &report_path)?;
    let scripts = slurm_scripts::discover_scripts(&paths)?
        .into_iter()
        .map(|path| {
            let body = slurm_scripts::read_body(&path)?;
            Ok(inspect_benchmark_local_slurm_script_body(paths.display(&path), &body))
        })
        .collect::<Result<Vec<_>>>()?;
    let report = build_benchmark_local_slurm_script_body_report(
        paths.display(&paths.root),
        paths.display(&paths.report),
        scripts,
    )?;
    slurm_scripts::publish(&paths, &report)?;
    ensure!(report.ok, "slurm script body validation failed; see {}", report.report_path);
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/readiness/slurm_scripts.rs"]
mod tests;
