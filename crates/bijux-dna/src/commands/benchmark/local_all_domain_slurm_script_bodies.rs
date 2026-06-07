use std::path::PathBuf;

use anyhow::{Context, Result};

use super::local_all_domain_slurm_scripts::{
    render_all_domain_slurm_scripts, DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT,
};
use super::local_slurm_script_bodies::validate_slurm_script_bodies;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_SLURM_SCRIPT_BODY_REPORT_PATH: &str =
    "target/slurm-dry-run/all-domains/no-placeholder-report.json";

pub(crate) fn run_validate_all_domain_slurm_script_bodies(
    args: &parse::BenchLocalValidateAllDomainSlurmScriptBodiesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let root_path =
        args.root.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT));
    let report_path = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_SCRIPT_BODY_REPORT_PATH));

    render_all_domain_slurm_scripts(&repo_root, root_path.clone())?;
    let report = validate_slurm_script_bodies(&repo_root, root_path, report_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_path);
    }
    Ok(())
}
