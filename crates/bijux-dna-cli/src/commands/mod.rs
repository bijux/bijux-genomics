#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::run::{load_manifests, load_profile, resolve_run_base_dir};
use bijux_dna_api::v1::api::run::{CategorizedError, ErrorCategory};
use clap::Parser;

struct CwdGuard(PathBuf);

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

pub(crate) mod bam;
pub(crate) mod bench;
pub mod cli;
pub(crate) mod command_prelude;
pub(crate) mod fastq;
pub(crate) mod report_inputs;
pub(crate) mod run_plan;
pub(crate) mod validation;

include!("policies.rs");

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_args(args: &[&str], cwd: &Path) -> Result<()> {
    let cli = cli::Cli::parse_from(args);
    run_with_cli(&cli, cwd)
}

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_cli(cli: &cli::Cli, cwd: &Path) -> Result<()> {
    let original_cwd = std::env::current_dir().context("resolve current dir")?;
    std::env::set_current_dir(cwd).context("set current dir")?;
    let _guard = CwdGuard(original_cwd);

    if let Some(path) = &cli.telemetry_jsonl {
        let telemetry_path = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        std::env::set_var("BIJUX_TELEMETRY_JSONL", telemetry_path);
    }
    let domain_dir = cwd.join("domain");

    let dna_command = match &cli.command {
        cli::RootCommand::Dna { command } => command,
    };

    if fastq::handle_meta_commands(cli, dna_command, &domain_dir)? {
        return Ok(());
    }

    let profile_path = cwd
        .join("configs")
        .join("profiles")
        .join(format!("{}.toml", cli.profile));
    let mut profile = load_profile(&profile_path).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::PlanError,
            format!("failed to load profile {}: {err}", profile_path.display())
        ))
    })?;
    profile.run_base_dir = resolve_run_base_dir(cwd, &profile.run_base_dir);
    if cli.print_effective_config || cli.dump_effective_config {
        let payload = serde_json::json!({
            "profile": profile,
            "platform": cli.platform,
        });
        cli::render::json::print_pretty(&payload)?;
        return Ok(());
    }

    let registry = load_manifests(&domain_dir).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::ContractError,
            format!("manifest validation failed: {err}")
        ))
    })?;

    if bench::handle_fastq_bench(cli, dna_command, &registry)? {
        return Ok(());
    }

    if bam::handle_bam_commands(cli, dna_command, &registry, &domain_dir)? {
        return Ok(());
    }

    run_plan::run_plan(cli, dna_command, &registry, &domain_dir)
}
