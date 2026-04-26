use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_compiler::{
    compile_domain_configs, CompileOptions, DEFAULT_COMPILE_SCOPE, DEFAULT_CONFIGS_DIR,
    DEFAULT_DOMAIN_DIR,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "compile_domain_configs")]
struct Args {
    #[arg(long, default_value = DEFAULT_DOMAIN_DIR)]
    domain_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_CONFIGS_DIR)]
    configs_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_COMPILE_SCOPE)]
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
