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
    Cycle {
        operation: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_cycle_uses_the_direct_operator_surface() {
        let cli = Cli::try_parse_from([
            "bijux-dna-dev",
            "containers",
            "cycle",
            "plan",
            "registry.toml",
            "runs",
            "submissions",
            "2026-08-09T12:00:00Z",
        ])
        .unwrap_or_else(|error| panic!("parse direct container cycle command: {error}"));
        let Command::Containers(command) = cli.command else {
            panic!("expected containers command");
        };
        let ContainersSubcommand::Cycle { operation, args } = command.command else {
            panic!("expected container cycle command");
        };
        assert_eq!(operation, "plan");
        assert_eq!(args.len(), 4);
    }
}
