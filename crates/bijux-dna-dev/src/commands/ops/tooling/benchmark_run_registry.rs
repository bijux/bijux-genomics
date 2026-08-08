use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_run_registry(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkRunRegistryWriteMode::Write,
        [flag] if flag == "--check" => bijux_dna_bench::BenchmarkRunRegistryWriteMode::Check,
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-run-registry -- [--check]\n",
            ));
        }
    };
    let registry =
        bijux_dna_bench::publish_checked_in_benchmark_run_registry(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "benchmark run registry: {} total, {} active, {} reserved, next run-{:04}\n",
        registry.summary.run_count,
        registry.summary.active_count,
        registry.summary.reserved_count,
        registry.namespace.next_run_number
    )))
}
