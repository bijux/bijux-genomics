use anyhow::{anyhow, Context, Result};
use bijux_api::v1::run::{load_manifests, load_profile, normalize_run_base_dir};
use bijux_core::{CategorizedError, ErrorCategory};
use clap::Parser;

use crate::commands::{handle_bam_commands, handle_fastq_bench, handle_meta_commands, run_plan};
use crate::commands::cli::Cli;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(exit_code_for_error(&err));
    }
}

fn exit_code_for_error(err: &anyhow::Error) -> i32 {
    if let Some(category) = error_category_from_chain(err) {
        return match category {
            ErrorCategory::UserError => 2,
            ErrorCategory::DataError => 3,
            ErrorCategory::ToolError => 4,
            ErrorCategory::InfraError => 5,
            ErrorCategory::Bug => 70,
        };
    }
    let msg = err.to_string().to_lowercase();
    if msg.contains("invalid arg") || msg.contains("usage:") {
        2
    } else if msg.contains("invalid") || msg.contains("missing") || msg.contains("not found") {
        3
    } else if msg.contains("tool") && msg.contains("failed") {
        4
    } else if msg.contains("contract") || msg.contains("invariant") {
        5
    } else {
        70
    }
}

fn error_category_from_chain(err: &anyhow::Error) -> Option<ErrorCategory> {
    if let Some(categorized) = err.downcast_ref::<CategorizedError>() {
        return Some(categorized.category);
    }
    for cause in err.chain() {
        if let Some(categorized) = cause.downcast_ref::<CategorizedError>() {
            return Some(categorized.category);
        }
    }
    None
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("resolve current directory")?;
    if let Some(path) = &cli.telemetry_jsonl {
        let telemetry_path = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        std::env::set_var("BIJUX_TELEMETRY_JSONL", telemetry_path);
    }
    let domain_dir = cwd.join("domain");

    if handle_meta_commands(&cli, &domain_dir)? {
        return Ok(());
    }

    let profile_path = cwd
        .join("configs")
        .join("profiles")
        .join(format!("{}.toml", cli.profile));
    let mut profile = load_profile(&profile_path).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::UserError,
            format!("failed to load profile {}: {err}", profile_path.display())
        ))
    })?;
    profile.run_base_dir = normalize_run_base_dir(&cwd, &profile.run_base_dir);
    if cli.print_effective_config || cli.dump_effective_config {
        let payload = serde_json::json!({
            "profile": profile,
            "platform": cli.platform,
        });
        render::json::print_pretty(&payload)?;
        return Ok(());
    }

    let registry = load_manifests(&domain_dir).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::DataError,
            format!("manifest validation failed: {err}")
        ))
    })?;

    if handle_fastq_bench(&cli, &registry)? {
        return Ok(());
    }

    if handle_bam_commands(&cli, &registry, &domain_dir)? {
        return Ok(());
    }

    run_plan(&cli, &registry, &domain_dir)
}
