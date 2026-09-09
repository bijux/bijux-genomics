use super::all_domain_rendered_commands::{
    render_all_domain_commands, DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::AllDomainNoPlaceholderCommandCheckReport;
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/no-placeholder-command-check.json";

pub(crate) fn run_render_all_domain_no_placeholder_command_check(
    args: &parse::BenchReadinessRenderAllDomainNoPlaceholderCommandCheckArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_no_placeholder_command_check(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_no_placeholder_command_check(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainNoPlaceholderCommandCheckReport> {
    let observed = render_all_domain_commands(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
    )?;
    bijux_dna_api::v1::api::bench::write_recorded_command_audit(repo_root, &output_path, &observed)
}
