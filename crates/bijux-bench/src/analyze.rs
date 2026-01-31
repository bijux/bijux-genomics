pub use bijux_analyze::{build_rankings, print_rank_explain, RankInput, RankingEntry};

use crate::artifacts::{write_observations_jsonl, write_summary_json};
use crate::contract::{BenchmarkDecision, BenchmarkSummary};
use crate::model::BenchObservation;
use crate::policy::GatePolicy;
use crate::stats::bootstrap_ci;

/// Build a summary and emit deterministic bench artifacts.
///
/// # Errors
/// Returns an error if artifacts cannot be written.
pub fn write_bench_artifacts(
    base_dir: &std::path::Path,
    suite_id: &str,
    dataset_hash: &str,
    observations: &[BenchObservation],
    policy: &GatePolicy,
) -> anyhow::Result<()> {
    let observations_path = base_dir.join("bench_observations.jsonl");
    write_observations_jsonl(&observations_path, observations)?;

    let mut warnings = Vec::new();
    if observations.len() < 3 {
        warnings.push("low_sample_size".to_string());
    }
    let runtimes: Vec<f64> = observations.iter().map(|obs| obs.runtime_s).collect();
    let bootstrap = bootstrap_ci(&runtimes, 200, 7);
    if bootstrap.samples == 0 {
        warnings.push("bootstrap_disabled".to_string());
    }
    let _bootstrap_summary = (
        bootstrap.mean,
        bootstrap.ci_low,
        bootstrap.ci_high,
        bootstrap.samples,
    );

    let mut decisions = Vec::new();
    for obs in observations {
        let gate = policy.decide(&obs.metrics.payload);
        decisions.push(BenchmarkDecision {
            tool: obs.tool.clone(),
            passes: gate.passes,
            rationale: gate.trace,
        });
    }

    let summary = BenchmarkSummary::v1(
        suite_id.to_string(),
        dataset_hash.to_string(),
        observations.len(),
        warnings,
        decisions,
    );
    let summary_path = base_dir.join("bench_summary.json");
    write_summary_json(&summary_path, &summary)?;
    Ok(())
}
