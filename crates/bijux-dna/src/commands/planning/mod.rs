mod entrypoint;

use bijux_dna_api::v1::api::run::ToolRegistry;

use crate::commands::support::prelude::{Cli, DnaCommand, Path, Result};

pub(crate) fn run_plan(
    cli: &Cli,
    dna_command: &DnaCommand,
    registry: &ToolRegistry,
    domain_dir: &Path,
) -> Result<()> {
    entrypoint::run_plan(cli, dna_command, registry, domain_dir)
}
