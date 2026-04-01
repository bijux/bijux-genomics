//! Owner: bijux-dna-bench
//! Internal statistical support for benchmark summarization.

use bijux_dna_bench_model::stats::{bootstrap_ci, seed_from_ids};
use bijux_dna_bench_model::{BenchmarkObservation, BenchmarkSuiteSpec};

pub(super) fn bootstrap_if_enabled(
    suite: &BenchmarkSuiteSpec,
    stage_id: &str,
    tool_id: &str,
    metric_id: &str,
    values: &[f64],
    samples: Option<usize>,
) -> Option<(f64, f64)> {
    let samples = samples.unwrap_or(0);
    if samples == 0 || values.len() < 5 {
        return None;
    }
    let seed = seed_from_ids(&suite.suite_id, metric_id, stage_id, tool_id);
    let result = bootstrap_ci(values, samples, seed);
    Some((result.ci_low, result.ci_high))
}

pub(super) fn indices_to_replicates(
    indices: &[usize],
    group: &[&BenchmarkObservation],
) -> Vec<String> {
    let mut replicates = Vec::new();
    for idx in indices {
        if let Some(obs) = group.get(*idx) {
            replicates.push(obs.replicate_id.clone());
        }
    }
    replicates
}
