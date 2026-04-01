//! Owner: bijux-dna-bench
//! Internal grouping and statistical support for benchmark summarization.

use bijux_dna_bench_model::stats::{bootstrap_ci, seed_from_ids};
use bijux_dna_bench_model::{BenchmarkObservation, BenchmarkSuiteSpec};

pub(super) type StageDatasetScope = (String, String, Option<String>, Option<String>);
pub(super) type StageDatasetToolScope = (String, String, Option<String>, Option<String>, String);
pub(super) type SummaryGroupKey = (
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
);
pub(super) type SummaryStratumKey = (String, Option<String>, Option<String>, String);

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

pub(super) fn stage_scope_label(
    stage_id: &str,
    stage_instance_id: Option<&str>,
    lineage_id: Option<&str>,
    dataset_id: &str,
) -> String {
    let mut parts = vec![stage_id.to_string(), dataset_id.to_string()];
    if let Some(stage_instance_id) = stage_instance_id {
        parts.push(stage_instance_id.to_string());
    }
    if let Some(lineage_id) = lineage_id {
        parts.push(lineage_id.to_string());
    }
    parts.join(":")
}
