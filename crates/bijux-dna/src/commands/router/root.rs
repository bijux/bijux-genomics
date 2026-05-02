use std::path::Path;

use anyhow::Result;

use crate::commands::cli;
#[cfg(debug_assertions)]
use crate::commands::router::root_commands::handle_slurm_root;
#[cfg(debug_assertions)]
use crate::commands::router::root_commands::{
    handle_ci_root, handle_config_root, handle_domain_root, handle_ena_root, handle_lab_root,
    handle_tool_root,
};
use crate::commands::router::root_commands::{
    handle_corpus_root, handle_environment_root, handle_registry_root,
};
use crate::commands::status::handle_status_root;

/// # Errors
/// Returns an error if any routed root command fails.
pub(crate) fn try_handle_root_command(
    dna_command: &cli::DnaCommand,
    cwd: &Path,
    platform: Option<&str>,
) -> Result<bool> {
    match dna_command {
        cli::DnaCommand::Environment(args) => {
            handle_environment_root(&args.command, cwd, platform)?;
            Ok(true)
        }
        cli::DnaCommand::Registry(args) => {
            handle_registry_root(&args.command, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Ena(args) => {
            handle_ena_root(&args.command, cwd)?;
            Ok(true)
        }
        cli::DnaCommand::Corpus(args) => {
            handle_corpus_root(&args.command, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Tool(args) => {
            handle_tool_root(&args.command, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Domain(args) => {
            handle_domain_root(&args.command, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Lab(args) => {
            handle_lab_root(&args.command, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Config(args) => {
            handle_config_root(&args.command, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Slurm(args) => {
            handle_slurm_root(&args.command, cwd)?;
            Ok(true)
        }
        cli::DnaCommand::Status(args) => {
            handle_status_root(args, cwd)?;
            Ok(true)
        }
        #[cfg(debug_assertions)]
        cli::DnaCommand::Ci(args) => {
            handle_ci_root(&args.command, cwd)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
