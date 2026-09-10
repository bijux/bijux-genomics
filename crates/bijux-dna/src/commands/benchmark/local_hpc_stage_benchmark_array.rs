use super::local_hpc_execution_resolver::collect_hpc_execution_resolver_report_for_jobs;
use super::local_hpc_scratch_layout::collect_hpc_scratch_layout_for_jobs;
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    collect_hpc_stage_array, publish_hpc_stage_array, validate_hpc_stage_array,
    BenchmarkLocalHpcStageBenchmarkArrayReport, HpcStageArrayDestination, HpcStageArrayRequest,
};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/slurm/stage-benchmark-array.sbatch";
pub(crate) type LocalHpcStageBenchmarkArrayReport = BenchmarkLocalHpcStageBenchmarkArrayReport;
pub(crate) fn run_render_hpc_stage_benchmark_array(
    args: &parse::BenchLocalRenderHpcStageBenchmarkArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH))
    });
    let report = render_hpc_stage_benchmark_array(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_stage_benchmark_array(
    args: &parse::BenchLocalValidateHpcStageBenchmarkArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let script_path = args.script.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH))
    });
    let report = validate_hpc_stage_benchmark_array_path(&repo_root, &script_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_stage_benchmark_array(
    root: &Path,
    path: PathBuf,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    with_observations(root, &path, publish_hpc_stage_array)
}
pub(crate) fn collect_hpc_stage_benchmark_array(
    root: &Path,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    with_observations(
        root,
        Path::new(DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH),
        collect_hpc_stage_array,
    )
}
pub(crate) fn validate_hpc_stage_benchmark_array_path(
    root: &Path,
    path: &Path,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    with_observations(root, path, validate_hpc_stage_array)
}
fn with_observations(
    root: &Path,
    path: &Path,
    service: impl FnOnce(HpcStageArrayRequest<'_>) -> Result<LocalHpcStageBenchmarkArrayReport>,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    let destination = HpcStageArrayDestination::prepare(root, path)?;
    let jobs = load_local_hpc_selected_jobs(root)?;
    let scratch = collect_hpc_scratch_layout_for_jobs(root, &jobs)?;
    let executions = collect_hpc_execution_resolver_report_for_jobs(root, &jobs)?;
    service(HpcStageArrayRequest {
        repo_root: root,
        destination,
        jobs: &jobs,
        scratch: &scratch.jobs,
        executions: &executions.rows,
    })
}
