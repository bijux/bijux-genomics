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
    let registry_path = cwd.join("configs").join("tool_registry.toml");

    if let cli::RootCommand::Environment { command } = &cli.command {
        return handle_environment_root(command, cwd);
    }
    if let cli::RootCommand::Registry { command } = &cli.command {
        return handle_registry_root(command, cwd);
    }
    let dna_command = match &cli.command {
        cli::RootCommand::Dna { command } => command,
        cli::RootCommand::Environment { .. } | cli::RootCommand::Registry { .. } => {
            unreachable!("handled above")
        }
    };

    if fastq::handle_meta_commands(cli, dna_command, &domain_dir, &registry_path)? {
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

    let registry = load_manifests(&registry_path).map_err(|err| {
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

fn handle_environment_root(command: &cli::EnvCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        env_doctor, print_env_images, print_env_info, print_env_registry_list, run_env_smoke,
    };
    use bijux_dna_api::v1::api::env::{load_image_catalog, load_platform};
    match command {
        cli::EnvCommand::List => {
            let registry_path = cwd.join("configs").join("tool_registry.toml");
            print_env_registry_list(&registry_path)?;
        }
        cli::EnvCommand::Smoke(args) => {
            run_env_smoke(&args.runtime, &args.tool)?;
        }
        cli::EnvCommand::Images | cli::EnvCommand::Info | cli::EnvCommand::Doctor => {
            let platform =
                load_platform(None).map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                cli::EnvCommand::Images => print_env_images(&catalog, &platform)?,
                cli::EnvCommand::Info => print_env_info(&catalog, &platform),
                cli::EnvCommand::Doctor => env_doctor(&catalog, &platform),
                cli::EnvCommand::List | cli::EnvCommand::Smoke(_) => {}
            }
        }
    }
    Ok(())
}

fn handle_registry_root(command: &cli::RegistryCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        print_registry_list_stages, print_registry_list_tools, print_registry_show,
    };
    let registry_path = cwd.join("configs").join("tool_registry.toml");
    match command {
        cli::RegistryCommand::ListTools => print_registry_list_tools(&registry_path)?,
        cli::RegistryCommand::ListStages => print_registry_list_stages(&registry_path)?,
        cli::RegistryCommand::Show { id } => print_registry_show(&registry_path, id)?,
    }
    Ok(())
}
