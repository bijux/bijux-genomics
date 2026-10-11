//! CLI request adaptation and presentation for model-admitted failure simulation.
use super::path_resolution::BenchmarkPathResolver;
use super::readiness::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::readiness::all_domain_output_declarations::collect_all_domain_output_declaration_rows;
use super::readiness::all_domain_rendered_commands::collect_all_domain_rendered_command_rows;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::fake_run_all_domain_failure_results;
pub(crate) use bijux_dna_api::v1::api::bench::{
    AllDomainFakeFailuresManifest, DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_fake_run_all_domain_failures(
    args: &parse::BenchLocalFakeRunAllDomainFailuresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest = fake_run_all_domain_failures(
        &repo_root,
        args.output_root.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_local_fake_run_root().join("all-domains-failures")
        }),
        args.exit_code,
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.failure_root);
    }
    Ok(())
}

pub(crate) fn fake_run_all_domain_failures(
    repo_root: &Path,
    output_root: PathBuf,
    exit_code: i32,
) -> Result<AllDomainFakeFailuresManifest> {
    let expected = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let commands = collect_all_domain_rendered_command_rows(repo_root)?;
    let outputs = collect_all_domain_output_declaration_rows(repo_root)?;
    fake_run_all_domain_failure_results(
        repo_root,
        &output_root,
        &expected,
        &commands,
        &outputs,
        exit_code,
    )
}
