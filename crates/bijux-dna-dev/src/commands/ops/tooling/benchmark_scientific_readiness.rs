use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_scientific_readiness(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkCoverageWriteMode::Write,
        [flag] if flag == "--check" => bijux_dna_bench::BenchmarkCoverageWriteMode::Check,
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-scientific-readiness -- [--check]\n",
            ));
        }
    };
    let report = bijux_dna_bench::publish_checked_in_scientific_readiness(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "scientific benchmark readiness: {} ready, {} blocked\n",
        report.summary.ready_count, report.summary.blocked_count
    )))
}
