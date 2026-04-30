use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_compiler::{
    build_domain_registry_bundle, load_domain_registry_bundle, write_domain_registry_bundle,
    DEFAULT_CONFIGS_DIR, DEFAULT_DOMAIN_DIR,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "domain_registry_bundle")]
struct Args {
    #[arg(long, default_value = DEFAULT_DOMAIN_DIR)]
    domain_dir: PathBuf,
    #[arg(long, default_value = DEFAULT_CONFIGS_DIR)]
    configs_dir: PathBuf,
    #[arg(long)]
    bundle: Option<PathBuf>,
    #[arg(long)]
    write_generated: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bundle = if let Some(path) = args.bundle.as_deref() {
        load_domain_registry_bundle(path)?
    } else {
        build_domain_registry_bundle(&args.domain_dir, "workspace-local")?
    };
    if args.write_generated {
        let _ = write_domain_registry_bundle(&args.configs_dir, &bundle)?;
    }
    println!("{}", serde_json::to_string_pretty(&bundle)?);
    Ok(())
}
