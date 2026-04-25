use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = bijux_dna_science::cli::ScienceCli::parse();
    bijux_dna_science::app::run(cli)
}
