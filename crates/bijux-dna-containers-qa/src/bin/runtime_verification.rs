use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "runtime_verification", version, about = "Verify catalog-locked Apptainer images")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute bounded version and help probes for every current retained image.
    Execute {
        repository_root: PathBuf,
        study_root: PathBuf,
        inventory: PathBuf,
        apptainer: PathBuf,
        data_root: PathBuf,
        evidence_root: PathBuf,
        json_output: PathBuf,
        markdown_output: PathBuf,
        #[arg(long)]
        baseline: Option<PathBuf>,
        workers: usize,
        timeout_seconds: u64,
    },
    /// Verify that a Markdown report exactly renders its typed publication.
    CheckReport { json_output: PathBuf, markdown_output: PathBuf },
}

fn main() -> Result<()> {
    match Args::parse().command {
        Command::Execute {
            repository_root,
            study_root,
            inventory,
            apptainer,
            data_root,
            evidence_root,
            json_output,
            markdown_output,
            baseline,
            workers,
            timeout_seconds,
        } => {
            let result = bijux_dna_containers_qa::api::execute_runtime_verification(
                &bijux_dna_containers_qa::api::RuntimeVerificationRequest {
                    repository_root,
                    study_root,
                    inventory_path: inventory,
                    apptainer_executable: apptainer,
                    data_root,
                    evidence_root,
                    baseline_publication_path: baseline,
                    worker_count: workers,
                    timeout: Duration::from_secs(timeout_seconds),
                },
                &json_output,
                &markdown_output,
            )?;
            if result.failed != 0 {
                anyhow::bail!(
                    "runtime verification completed with {} passed and {} failed tools",
                    result.passed,
                    result.failed
                );
            }
            println!(
                "runtime verification completed: {} passed, 0 failed, {} executed, {} reused",
                result.passed, result.executed, result.reused
            );
            Ok(())
        }
        Command::CheckReport { json_output, markdown_output } => {
            bijux_dna_containers_qa::api::verify_runtime_verification_report(
                &json_output,
                &markdown_output,
            )
        }
    }
}
