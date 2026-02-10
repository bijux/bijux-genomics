use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_compiler::{validate_domain, ValidateOptions};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "domain_validate")]
struct Args {
    #[arg(long, default_value = "domain")]
    domain_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    validate_domain(&ValidateOptions {
        domain_dir: args.domain_dir,
    })
}
