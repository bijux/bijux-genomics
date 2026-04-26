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
    let min_replicates = usize::try_from(suite.analysis_requirements.min_replicates_for_bootstrap)
        .unwrap_or(usize::MAX);
    if samples == 0 || values.len() < min_replicates {
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

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::{
        AnalysisRequirements, BenchmarkSuiteSpec, DatasetSpec, DiversityRequirements,
        ReplicatePolicy, StratificationRequirement,
    };

    use super::bootstrap_if_enabled;

    fn suite(min_replicates_for_bootstrap: u32) -> BenchmarkSuiteSpec {
        BenchmarkSuiteSpec::v1(
            "suite-1".to_string(),
            vec![DatasetSpec {
                id: "dataset-1".to_string(),
                hash: "hash-1".to_string(),
                size: 100,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            &["fastq.trim_reads".to_string()],
            &["fastp".to_string()],
            &["params-a".to_string()],
            ReplicatePolicy { count: 6, warmup: 0, seeds: vec![1, 2, 3, 4, 5, 6] },
            DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: true,
                require_outlier_detection: true,
                min_replicates_for_bootstrap,
            },
        )
    }

    #[test]
    fn bootstrap_respects_suite_minimum_replicates() {
        let values = [1.0, 1.1, 0.9, 1.2, 1.0];

        let ci = bootstrap_if_enabled(
            &suite(6),
            "fastq.trim_reads",
            "fastp",
            "runtime_s",
            &values,
            Some(10),
        );

        assert!(ci.is_none());
    }
}
