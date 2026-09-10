use super::path_resolution::BenchmarkPathResolver;
use super::readiness::{
    all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows,
    all_domain_output_declarations::collect_all_domain_output_declaration_rows,
    all_domain_rendered_commands::collect_all_domain_rendered_command_rows,
};
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::{
    all_domain_simulation_output_ids as declared_output_ids,
    all_domain_simulation_output_path as output_relative_path, AllDomainFakeRunResultReport,
    AllDomainFakeRunsReport, DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_fake_run_all_domains(
    args: &parse::BenchLocalFakeRunAllDomainsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest = fake_run_all_domain_benchmark_results(
        &repo_root,
        args.output_root
            .clone()
            .unwrap_or_else(|| benchmark_paths.benchmark_local_fake_run_root().join("all-domains")),
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.fake_run_root);
    }
    Ok(())
}

pub(crate) fn fake_run_all_domain_benchmark_results(
    repo_root: &Path,
    output_root: PathBuf,
) -> Result<AllDomainFakeRunsReport> {
    let expected = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let commands = collect_all_domain_rendered_command_rows(repo_root)?;
    let outputs = collect_all_domain_output_declaration_rows(repo_root)?;
    bijux_dna_api::v1::api::bench::fake_run_all_domain_benchmark_results(
        repo_root,
        &output_root,
        &expected,
        &commands,
        &outputs,
    )
}
