use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    build_benchmark_local_hpc_dependency_simulation, build_benchmark_local_hpc_job_graph,
    BenchmarkLocalHpcDependencySimulationReport,
};

use super::local_all_domain_slurm_submit_manifest::{
    BenchLocalAllDomainSlurmSubmitJob, DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
};
use super::local_hpc_job_graph::{collect_local_hpc_job_graph, LocalHpcJobGraph};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/slurm-dependency-simulation.json";

pub(crate) type LocalHpcDependencySimulationReport = BenchmarkLocalHpcDependencySimulationReport;

pub(crate) fn run_render_hpc_dependency_simulation(
    args: &parse::BenchLocalRenderHpcDependencySimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH))
    });
    let report = render_hpc_dependency_simulation(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_dependency_simulation(
    args: &parse::BenchLocalValidateHpcDependencySimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH))
    });
    let report = validate_hpc_dependency_simulation_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_dependency_simulation(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcDependencySimulationReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_dependency_simulation(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)?;
    Ok(report)
}

pub(crate) fn collect_hpc_dependency_simulation_with_jobs<
    J: AsRef<[BenchLocalAllDomainSlurmSubmitJob]>,
>(
    repo_root: &Path,
    acquire_jobs: impl FnOnce() -> Result<J>,
) -> Result<LocalHpcDependencySimulationReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path =
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH));
    build_hpc_dependency_simulation_with_graph(repo_root, &output_path, || {
        let jobs = acquire_jobs()?;
        build_benchmark_local_hpc_job_graph(
            DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
            jobs.as_ref().to_vec(),
        )
    })
}

pub(crate) fn validate_hpc_dependency_simulation_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcDependencySimulationReport> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = build_hpc_dependency_simulation(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected = serde_json::to_vec_pretty(&report)
        .context("serialize governed HPC dependency simulation report")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC dependency simulation `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-dependency-simulation --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    Ok(report)
}

fn build_hpc_dependency_simulation(
    repo_root: &Path,
    absolute_output_path: &Path,
) -> Result<LocalHpcDependencySimulationReport> {
    build_hpc_dependency_simulation_with_graph(repo_root, absolute_output_path, || {
        collect_local_hpc_job_graph(repo_root)
    })
}

fn build_hpc_dependency_simulation_with_graph(
    repo_root: &Path,
    absolute_output_path: &Path,
    acquire_graph: impl FnOnce() -> Result<LocalHpcJobGraph>,
) -> Result<LocalHpcDependencySimulationReport> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output_path,
        "HPC dependency simulation output",
    )?;
    let graph = acquire_graph()?;
    build_benchmark_local_hpc_dependency_simulation(
        &graph,
        &path_relative_to_repo(repo_root, absolute_output_path),
    )
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}
