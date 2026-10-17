mod adapters;
use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::BenchmarkLocalHpcCandidateRunManifest;
use std::path::{Path, PathBuf};
pub(crate) type LocalHpcCandidateRunManifest = BenchmarkLocalHpcCandidateRunManifest;
pub(crate) const DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH: &str =
    "artifacts/benchmark-smoke/readiness/hpc/FIRST_HPC_CANDIDATE_RUN.json";
pub(crate) fn run_render_hpc_candidate_run_manifest(
    args: &parse::BenchLocalRenderHpcCandidateRunManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("FIRST_HPC_CANDIDATE_RUN.json")
    });
    let report = render_hpc_candidate_run_manifest(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn run_validate_hpc_candidate_run_manifest(
    args: &parse::BenchLocalValidateHpcCandidateRunManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("FIRST_HPC_CANDIDATE_RUN.json")
    });
    let report = validate_hpc_candidate_run_manifest_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_hpc_candidate_run_manifest(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcCandidateRunManifest> {
    bijux_dna_bench_hpc::candidate::render_hpc_candidate_run_manifest(
        &adapters::context(repo_root),
        output_path,
    )
}
pub(crate) fn validate_hpc_candidate_run_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcCandidateRunManifest> {
    bijux_dna_bench_hpc::candidate::validate_hpc_candidate_run_manifest_path(
        &adapters::context(repo_root),
        manifest_path,
    )
}
pub(crate) fn collect_hpc_candidate_run_manifest_with_jobs<'jobs>(
    repo_root: &Path,
    selected_jobs: &dyn Fn() -> Result<&'jobs [BenchLocalAllDomainSlurmSubmitJob]>,
) -> Result<LocalHpcCandidateRunManifest> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path =
        benchmark_paths.benchmark_hpc_readiness_root().join("FIRST_HPC_CANDIDATE_RUN.json");
    let evidence = adapters::SelectedJobsEvidence { selected_jobs };
    let context =
        bijux_dna_bench_hpc::candidate::CandidateContext { repo_root, evidence: &evidence };
    bijux_dna_bench_hpc::candidate::collect_hpc_candidate_run_manifest(&context, &output_path)
}
