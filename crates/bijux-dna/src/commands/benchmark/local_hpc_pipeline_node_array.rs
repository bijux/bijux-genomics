use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_scratch_layout::collect_hpc_scratch_layout_for_jobs;
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::local_pipeline_dag::{
    benchmark_local_pipeline_config_path, validate_pipeline_dag_path,
    LocalPipelineDagValidationReport,
};
use super::path_resolution::BenchmarkPathResolver;
use super::readiness::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    collect_hpc_pipeline_array, publish_hpc_pipeline_array, validate_hpc_pipeline_array,
    BenchmarkLocalHpcPipelineNodeArrayReport, HpcPipelineArrayDestination, HpcPipelineArrayRequest,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/slurm/pipeline-node-array.sbatch";
pub(crate) type LocalHpcPipelineNodeArrayReport = BenchmarkLocalHpcPipelineNodeArrayReport;
pub(crate) fn run_render_hpc_pipeline_node_array(
    args: &parse::BenchLocalRenderHpcPipelineNodeArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH))
    });
    let report = render_hpc_pipeline_node_array(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_pipeline_node_array(
    args: &parse::BenchLocalValidateHpcPipelineNodeArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let script_path = args.script.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH))
    });
    let report = validate_hpc_pipeline_node_array_path(&repo_root, &script_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_pipeline_node_array(
    root: &Path,
    path: PathBuf,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    with_observations(
        root,
        &path,
        || load_local_hpc_selected_jobs(root),
        publish_hpc_pipeline_array,
    )
}
pub(crate) fn collect_hpc_pipeline_node_array_with_jobs<
    J: AsRef<[BenchLocalAllDomainSlurmSubmitJob]>,
>(
    root: &Path,
    acquire_jobs: impl FnOnce() -> Result<J>,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    with_observations(
        root,
        Path::new(DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH),
        acquire_jobs,
        collect_hpc_pipeline_array,
    )
}
pub(crate) fn validate_hpc_pipeline_node_array_path(
    root: &Path,
    path: &Path,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    with_observations(
        root,
        path,
        || load_local_hpc_selected_jobs(root),
        validate_hpc_pipeline_array,
    )
}
fn with_observations<J: AsRef<[BenchLocalAllDomainSlurmSubmitJob]>>(
    root: &Path,
    path: &Path,
    acquire_jobs: impl FnOnce() -> Result<J>,
    service: impl FnOnce(HpcPipelineArrayRequest<'_>) -> Result<LocalHpcPipelineNodeArrayReport>,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    let destination = HpcPipelineArrayDestination::prepare(root, path)?;
    let jobs = acquire_jobs()?;
    let jobs = jobs.as_ref();
    let scratch = collect_hpc_scratch_layout_for_jobs(root, jobs)?;
    let pipelines = load_pipeline_reports(root)?;
    service(HpcPipelineArrayRequest {
        repo_root: root,
        destination,
        jobs,
        scratch: &scratch.jobs,
        pipelines: &pipelines,
        pipeline_order: ESSENTIAL_PIPELINE_IDS,
    })
}
fn load_pipeline_reports(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalPipelineDagValidationReport>> {
    let mut reports = BTreeMap::new();
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path = benchmark_local_pipeline_config_path(repo_root, pipeline_id);
        let report_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag")
            .join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        reports.insert((*pipeline_id).to_string(), report);
    }
    Ok(reports)
}
