mod entrypoint;

use crate::commands::cli;
use crate::commands::support::prelude::{Path, Result};

pub(crate) fn handle_status_root(args: &cli::StatusArgs, cwd: &Path) -> Result<()> {
    entrypoint::handle_status_root(args, cwd)
}
