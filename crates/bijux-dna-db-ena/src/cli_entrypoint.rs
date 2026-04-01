#[path = "cli/mod.rs"]
mod cli;

pub(super) fn run() -> anyhow::Result<()> {
    cli::run()
}
