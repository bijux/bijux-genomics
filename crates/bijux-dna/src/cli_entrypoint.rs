use std::path::Path;

use anyhow::Context;
use anyhow::Result;

use crate::commands;

/// # Errors
/// Returns an error if the current directory cannot be resolved or CLI execution fails.
pub fn run_from_env() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    let cli = commands::parse_process_cli(&argv);
    let cwd = std::env::current_dir().context("resolve current directory")?;
    commands::run_with_cli(&cli, &cwd)
}

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_from_args(args: &[&str], cwd: &Path) -> Result<()> {
    commands::run_with_args(args, cwd)
}
