use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::application::checks::CheckApplication;
use crate::application::containers::ContainerApplication;
use crate::application::domain::DomainApplication;
use crate::model::check::{CheckSelection, CheckStatus};

#[derive(Parser, Debug)]
#[command(
    name = "bijux-dev-dna",
    about = "Versioned development control-plane for the Bijux DNA workspace"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Checks(ChecksCommand),
    Containers(ContainersCommand),
    Domain(DomainCommand),
}

#[derive(Parser, Debug)]
pub struct ChecksCommand {
    #[command(subcommand)]
    command: ChecksSubcommand,
}

#[derive(Subcommand, Debug)]
enum ChecksSubcommand {
    List,
    Run {
        #[arg(long, conflicts_with = "id")]
        all: bool,
        id: Option<String>,
    },
}

#[derive(Parser, Debug)]
pub struct ContainersCommand {
    #[command(subcommand)]
    command: ContainersSubcommand,
}

#[derive(Subcommand, Debug)]
enum ContainersSubcommand {
    List,
    Run {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
pub struct DomainCommand {
    #[command(subcommand)]
    command: DomainSubcommand,
}

#[derive(Subcommand, Debug)]
enum DomainSubcommand {
    List,
    Run {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// # Errors
/// Returns an error if CLI parsing or command execution fails.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Checks(command) => run_checks(command),
        Command::Containers(command) => run_containers(command),
        Command::Domain(command) => run_domain(command),
    }
}

fn run_checks(command: ChecksCommand) -> Result<()> {
    let app = CheckApplication::new()?;
    match command.command {
        ChecksSubcommand::List => {
            for check in app.registry() {
                println!("{}", check.id);
            }
            Ok(())
        }
        ChecksSubcommand::Run { all, id } => {
            let selection = if all {
                CheckSelection::All
            } else {
                CheckSelection::Single(
                    id.ok_or_else(|| anyhow::anyhow!("checks run requires <id> or --all"))?,
                )
            };
            let outcomes = app.run_selection(selection)?;
            let mut failed = false;
            for outcome in outcomes {
                println!(
                    "{}: {}",
                    outcome.id,
                    match outcome.status {
                        CheckStatus::Passed => "passed",
                        CheckStatus::Failed => {
                            failed = true;
                            "failed"
                        }
                    }
                );
                if outcome.status == CheckStatus::Failed && !outcome.detail.trim().is_empty() {
                    println!("{}", outcome.detail.trim());
                }
            }
            if failed {
                anyhow::bail!("one or more checks failed");
            }
            Ok(())
        }
    }
}

fn run_containers(command: ContainersCommand) -> Result<()> {
    let app = ContainerApplication::new()?;
    match command.command {
        ContainersSubcommand::List => {
            for command in app.registry()? {
                println!("{}", command.id);
            }
            Ok(())
        }
        ContainersSubcommand::Run { id, args } => {
            let outcome = app.run(&id, &args)?;
            if !outcome.stdout.is_empty() {
                print!("{}", outcome.stdout);
            }
            if !outcome.stderr.is_empty() {
                eprint!("{}", outcome.stderr);
            }
            if !outcome.is_success() {
                anyhow::bail!(
                    "container command `{id}` failed with exit code {}",
                    outcome.exit_code
                );
            }
            Ok(())
        }
    }
}

fn run_domain(command: DomainCommand) -> Result<()> {
    let app = DomainApplication::new()?;
    match command.command {
        DomainSubcommand::List => {
            for command in app.registry() {
                println!("{}", command.id);
            }
            Ok(())
        }
        DomainSubcommand::Run { id, args } => {
            let outcome = app.run(&id, &args)?;
            if !outcome.stdout.is_empty() {
                print!("{}", outcome.stdout);
            }
            if !outcome.stderr.is_empty() {
                eprint!("{}", outcome.stderr);
            }
            if !outcome.is_success() {
                anyhow::bail!(
                    "domain command `{id}` failed with exit code {}",
                    outcome.exit_code
                );
            }
            Ok(())
        }
    }
}
