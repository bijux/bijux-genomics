use crate::commands::cli::render;
#[cfg(not(feature = "bam_downstream"))]
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_bench_hpc::readiness::ReadinessContext;
pub(crate) use bijux_dna_bench_hpc::readiness::DEFAULT_HPC_SUBMISSION_READY_REPORT_PATH;
use std::path::{Path, PathBuf};
pub(crate) type BenchLocalHpcSubmissionReadyReport =
    bijux_dna_api::v1::api::bench::BenchmarkLocalHpcSubmissionReadinessReport;
mod adapters;
pub(crate) fn run_validate_hpc_submission_ready(output: Option<PathBuf>, json: bool) -> Result<()> {
    validate_hpc_submission_ready_feature_gate()?;

    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = validate_hpc_submission_ready(
        &repo_root,
        output.unwrap_or_else(|| PathBuf::from(DEFAULT_HPC_SUBMISSION_READY_REPORT_PATH)),
    )?;
    if json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn validate_hpc_submission_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BenchLocalHpcSubmissionReadyReport> {
    validate_hpc_submission_ready_feature_gate()?;
    let context = ReadinessContext {
        repo_root,
        backend: &adapters::LocalReadinessAdapter,
        paths: adapters::input_paths(),
    };
    bijux_dna_bench_hpc::readiness::validate_hpc_submission_ready(&context, output_path)
}
#[cfg(feature = "bam_downstream")]
fn validate_hpc_submission_ready_feature_gate() -> Result<()> {
    Ok(())
}
#[cfg(not(feature = "bam_downstream"))]
fn validate_hpc_submission_ready_feature_gate() -> Result<()> {
    Err(anyhow!(
        "local HPC submission readiness requires the `bam_downstream` feature; rerun with `cargo run -p bijux-dna --features bam_downstream -- bench local validate-hpc-submission-ready`"
    ))
}
