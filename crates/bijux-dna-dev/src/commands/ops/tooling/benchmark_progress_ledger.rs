//! Owner: bijux-dna-dev
//! Thin operator entry point for crate-owned progress-ledger validation.

use super::{path_from_arg, OpsCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-progress-ledger -- validate <ledger>\n";

pub(in super::super) fn tooling_benchmark_progress_ledger(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let [operation, path] = args else {
        return Ok(OpsCommandOutcome::failure(USAGE));
    };
    if operation != "validate" {
        return Ok(OpsCommandOutcome::failure(USAGE));
    }
    let ledger = bijux_dna_bench::load_benchmark_progress_ledger(&path_from_arg(workspace, path))?;
    Ok(OpsCommandOutcome::success(format!(
        "benchmark progress ledger verified: cycle {}, {} tasks, {} execution units, {} attempts, {} submitted or running units, {} collectible units\n",
        ledger.cycle_id,
        ledger.tasks.len(),
        ledger.execution_units.len(),
        ledger.attempts.len(),
        ledger.frontier.submitted_unit_ids.len(),
        ledger.frontier.collectible_unit_ids.len(),
    )))
}

#[cfg(test)]
mod tests {
    use super::USAGE;

    #[test]
    fn usage_names_the_bounded_validation_surface() {
        assert!(USAGE.contains("benchmark-progress-ledger -- validate <ledger>"));
    }
}
