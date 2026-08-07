use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_stage_universe(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkUniverseWriteMode::Write,
        [flag] if flag == "--check" => bijux_dna_bench::BenchmarkUniverseWriteMode::Check,
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-stage-universe -- [--check]\n",
            ))
        }
    };
    let universe = bijux_dna_bench::publish_checked_in_stage_universe(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "stage universe: {} total, {} benchmark required, {} qualification only, {} orchestration only\n",
        universe.summary.stage_count,
        universe.summary.benchmark_required_count,
        universe.summary.qualification_only_count,
        universe.summary.orchestration_only_count
    )))
}
