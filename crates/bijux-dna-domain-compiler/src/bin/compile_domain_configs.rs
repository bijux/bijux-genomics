use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_compiler::{compile_domain_configs, CompileOptions};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "compile_domain_configs")]
struct Args {
    #[arg(long, default_value = "domain")]
    domain_dir: PathBuf,
    #[arg(long, default_value = "configs")]
    configs_dir: PathBuf,
    #[arg(long, default_value = "pre_hpc_pre_vcf")]
    scope: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    compile_domain_configs(&CompileOptions {
        domain_dir: args.domain_dir,
        configs_dir: args.configs_dir,
        scope: args.scope,
    })
}
