use anyhow::{Context, Result};
use bijux_api::v1::api::run::{CategorizedError, ErrorCategory};
use clap::Parser;

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
            ErrorCategory::PlanError => 2,
            ErrorCategory::ContractError => 3,
            ErrorCategory::ParseError => 4,
            ErrorCategory::ToolError => 5,
            ErrorCategory::InfraError => 70,
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
    crate::commands::run_with_cli(&cli, &cwd)
}
