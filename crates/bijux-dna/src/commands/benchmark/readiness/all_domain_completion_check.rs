use super::{
    all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows,
    all_domain_output_declarations::collect_all_domain_output_declaration_rows,
    all_domain_rendered_commands::collect_all_domain_rendered_command_rows,
};
use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::{parse, render};
use anyhow::Result;
pub(crate) use bijux_dna_api::v1::api::bench::completion::{
    AllDomainCompletionCheckReport, DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_render_all_domain_completion_check(
    args: &parse::BenchReadinessRenderAllDomainCompletionCheckArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let report = render_all_domain_completion_check(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_readiness_root().join("completion-check-all-domains.json")
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_completion_check(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainCompletionCheckReport> {
    let expected = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let commands = collect_all_domain_rendered_command_rows(repo_root)?;
    let outputs = collect_all_domain_output_declaration_rows(repo_root)?;
    bijux_dna_api::v1::api::bench::render_all_domain_completion_check(
        repo_root,
        &output_path,
        &expected,
        &commands,
        &outputs,
    )
}
