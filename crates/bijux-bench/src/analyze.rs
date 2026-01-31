pub use bijux_analyze::{build_rankings, print_rank_explain, RankInput, RankingEntry};

use crate::artifacts::{write_observations_jsonl, write_summary_json};
use crate::contract::{BenchmarkDecision, BenchmarkSummary};
use crate::error::BenchError;
use crate::model::BenchObservation;
use crate::policy::GatePolicy;
use crate::stats::bootstrap_ci;

#[derive(Debug, Clone)]
pub struct BenchAnalyzeOptions {
    pub replicates: u32,
    pub ci_bootstrap: Option<usize>,
}

impl Default for BenchAnalyzeOptions {
    fn default() -> Self {
        Self {
            replicates: 1,
            ci_bootstrap: None,
        }
    }
}

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
    options: &BenchAnalyzeOptions,
) -> anyhow::Result<()> {
    let observations_path = base_dir.join("bench_observations.jsonl");
    for obs in observations {
        obs.validate().map_err(|err| {
            anyhow::anyhow!(BenchError::InvalidObservation {
                reason: err.to_string()
            })
        })?;
    }
    write_observations_jsonl(&observations_path, observations)?;

    let mut warnings = Vec::new();
    if options.replicates < 3 {
        warnings.push("low_power".to_string());
    }
    if observations.len() < 3 {
        warnings.push("low_sample_size".to_string());
    }
    let runtimes: Vec<f64> = observations.iter().map(|obs| obs.runtime_s).collect();
    let bootstrap_samples = options.ci_bootstrap.unwrap_or(0);
    if bootstrap_samples == 0 {
        warnings.push("bootstrap_disabled".to_string());
    }
    let bootstrap = if bootstrap_samples == 0 {
        bootstrap_ci(&[], 0, 7)
    } else {
        bootstrap_ci(&runtimes, bootstrap_samples, 7)
    };
    if bootstrap.samples == 0 && bootstrap_samples > 0 {
        warnings.push("bootstrap_failed".to_string());
    }
    let _bootstrap_summary = (
        bootstrap.mean,
        bootstrap.ci_low,
        bootstrap.ci_high,
        bootstrap.samples,
    );

    let mut decisions = Vec::new();
    for obs in observations {
        let gate = policy.decide(Some(&obs.metrics.stage_id), &obs.metrics.payload);
        decisions.push(BenchmarkDecision {
            tool: obs.tool.clone(),
            passes: gate.passes,
            missing_metrics: gate.missing_metrics.clone(),
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
