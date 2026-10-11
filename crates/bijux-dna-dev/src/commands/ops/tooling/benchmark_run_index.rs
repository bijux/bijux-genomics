use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_run_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkRunIndexWriteMode::Write,
        [flag] if flag == "--check" => bijux_dna_bench::BenchmarkRunIndexWriteMode::Check,
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-run-index -- [--check]\n",
            ));
        }
    };
    let index = bijux_dna_bench::publish_checked_in_benchmark_run_index(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "benchmark run index: {} registered, {} citable, {} unpublished\n",
        index.summary.registered_run_count,
        index.summary.citable_publication_count,
        index.summary.unpublished_run_count
    )))
}
