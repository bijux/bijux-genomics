//! Owner: bijux-dna-dev
//! Operator entry point for benchmark publication lifecycle classification.

use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_publication_registry(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkPublicationRegistryWriteMode::Write,
        [flag] if flag == "--check" => {
            bijux_dna_bench::BenchmarkPublicationRegistryWriteMode::Check
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-publication-registry -- [--check]\n",
            ));
        }
    };
    let registry =
        bijux_dna_bench::publish_checked_in_benchmark_publication_registry(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "benchmark publication registry verified: {} current products, {} current run reports, {} missing run reports, {} stale run reports, {} failed run reports, {} superseded run reports, {} active stage reports, {} historical stage reports, {} quarantined files\n",
        registry.summary.current_product_count,
        registry.summary.current_run_report_count,
        registry.summary.missing_run_report_count,
        registry.summary.stale_run_report_count,
        registry.summary.failed_run_report_count,
        registry.summary.superseded_run_report_count,
        registry.summary.active_stage_report_count,
        registry.summary.historical_stage_report_count,
        registry.summary.quarantined_file_count,
    )))
}
