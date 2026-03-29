use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "bijux-dna-dev",
    about = "Versioned development control-plane for the Bijux DNA workspace"
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Subcommand, Debug)]
pub(super) enum Command {
    Assets(OpsCommand),
    Checks(ChecksCommand),
    Containers(ContainersCommand),
    Domain(DomainCommand),
    Docs(OpsCommand),
    Examples(OpsCommand),
    Hpc(OpsCommand),
    Lab(OpsCommand),
    Smoke(OpsCommand),
    Test(OpsCommand),
    Tooling(OpsCommand),
}

#[derive(Parser, Debug)]
pub(super) struct ChecksCommand {
    #[command(subcommand)]
    pub(super) command: ChecksSubcommand,
}

#[derive(Subcommand, Debug)]
pub(super) enum ChecksSubcommand {
    List,
    Run {
        #[arg(long, conflicts_with = "id")]
        all: bool,
        id: Option<String>,
    },
}

#[derive(Parser, Debug)]
pub(super) struct ContainersCommand {
    #[command(subcommand)]
    pub(super) command: ContainersSubcommand,
}

#[derive(Subcommand, Debug)]
pub(super) enum ContainersSubcommand {
    List,
    Run {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Parser, Debug)]
pub(super) struct DomainCommand {
    #[command(subcommand)]
    pub(super) command: DomainSubcommand,
}

#[derive(Parser, Debug)]
pub(super) struct OpsCommand {
    #[command(subcommand)]
    pub(super) command: OpsSubcommand,
}

#[derive(Subcommand, Debug)]
pub(super) enum DomainSubcommand {
    List,
    Run {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(super) enum OpsSubcommand {
    List,
    Run {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}
