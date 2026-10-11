use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_stage_coverage(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkCoverageWriteMode::Write,
        [flag] if flag == "--check" => bijux_dna_bench::BenchmarkCoverageWriteMode::Check,
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-stage-coverage -- [--check]\n",
            ))
        }
    };
    let coverage = bijux_dna_bench::publish_checked_in_stage_coverage(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "stage benchmark campaigns: {} defined, {} admitted, {} stale, {} missing\n",
        coverage.summary.defined_stage_count,
        coverage.summary.admitted_stage_count,
        coverage.summary.stale_stage_count,
        coverage.summary.missing_stage_count
    )))
}
