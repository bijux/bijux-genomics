use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "security_verification", version, about = "Collect exact-image security probes")]
struct Args {
    repository_root: PathBuf,
    study_root: PathBuf,
    inventory: PathBuf,
    apptainer: PathBuf,
    scanner_contract: PathBuf,
    scanner_root: PathBuf,
    database_contract: PathBuf,
    database_root: PathBuf,
    evidence_root: PathBuf,
    publication: PathBuf,
    #[arg(long, default_value_t = 4)]
    workers: usize,
    #[arg(long, default_value_t = 900)]
    timeout_seconds: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let result = bijux_dna_containers_qa::api::execute_security_verification(
        &bijux_dna_containers_qa::api::SecurityVerificationRequest {
            repository_root: args.repository_root,
            study_root: args.study_root,
            inventory_path: args.inventory,
            apptainer_executable: args.apptainer,
            scanner_contract_path: args.scanner_contract,
            scanner_root: args.scanner_root,
            database_contract_path: args.database_contract,
            database_root: args.database_root,
            evidence_root: args.evidence_root,
            worker_count: args.workers,
            timeout: Duration::from_secs(args.timeout_seconds),
        },
        &args.publication,
    )?;
    if result.failed != 0 {
        anyhow::bail!(
            "security probe collection completed with {} collected and {} failed tools",
            result.collected,
            result.failed
        );
    }
    println!("security probe collection completed: {} collected, 0 failed", result.collected);
    Ok(())
}
