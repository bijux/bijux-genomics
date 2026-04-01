use anyhow::Result;
use crate::catalog::ops::{
    assets_registry, docs_registry, examples_registry, hpc_registry, lab_registry, smoke_registry,
    test_registry, tooling_registry,
};
use clap::Parser;

use super::command_dispatch::{run_checks, run_containers, run_domain, run_ops};
use super::schema::{
    Cli, Command,
};

pub(super) fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Assets(command) => run_ops("assets", command, assets_registry),
        Command::Checks(command) => run_checks(command),
        Command::Containers(command) => run_containers(command),
        Command::Domain(command) => run_domain(command),
        Command::Docs(command) => run_ops("docs", command, docs_registry),
        Command::Examples(command) => run_ops("examples", command, examples_registry),
        Command::Hpc(command) => run_ops("hpc", command, hpc_registry),
        Command::Lab(command) => run_ops("lab", command, lab_registry),
        Command::Smoke(command) => run_ops("smoke", command, smoke_registry),
        Command::Test(command) => run_ops("test", command, test_registry),
        Command::Tooling(command) => run_ops("tooling", command, tooling_registry),
    }
}
