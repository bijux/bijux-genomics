//! Owner: bijux-dna-dev
//! Thin operator entry point for crate-owned benchmark stage/tool closure publication.

use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_stage_tool_closure(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkStageToolClosureWriteMode::Write,
        [flag] if flag == "--check" => {
            bijux_dna_bench::BenchmarkStageToolClosureWriteMode::Check
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-stage-tool-closure -- [--check]\n",
            ))
        }
    };
    let closure =
        bijux_dna_bench::publish_checked_in_benchmark_stage_tool_closure(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "stage/tool closure: {} catalog, {} domain, {} aligned, {} drift\n",
        closure.summary.catalog_pair_count,
        closure.summary.domain_pair_count,
        closure.summary.aligned_pair_count,
        closure.summary.drift_pair_count
    )))
}
