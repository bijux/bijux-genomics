use std::collections::BTreeMap;

use bijux_dna_benchmark::{summarize, BenchRunOptions, BenchmarkSuiteSpec};
use bijux_dna_benchmark::{
    AnalysisRequirements, DatasetSpec, DiversityRequirements, ReplicatePolicy,
    StratificationRequirement,
};
use bijux_dna_benchmark::{BenchmarkObservation, MetricsEnvelope};

fn obs(
    run_id: &str,
    dataset_id: &str,
    stage_id: &str,
    tool_id: &str,
    params_hash: &str,
) -> BenchmarkObservation {
    BenchmarkObservation {
        schema_version: "bijux.bench.observation.v1".to_string(),
        run_id: run_id.to_string(),
        dataset_id: dataset_id.to_string(),
        dataset_class: "trueseq".to_string(),
        read_layout: "paired".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: "1.0".to_string(),
        image_digest: "sha256:abc".to_string(),
        container_digest: "sha256:abc".to_string(),
        params_hash: params_hash.to_string(),
        input_hash: "input".to_string(),
        runtime_s: 1.0,
        memory_mb: 10.0,
        exit_code: 0,
        failure_kind: None,
        metrics: MetricsEnvelope {
            stage_id: stage_id.to_string(),
            schema_version: "metrics.v1".to_string(),
            values: BTreeMap::new(),
        },
        replicate_id: "r1".to_string(),
        replicate_index: 0,
        warmup_policy: "none".to_string(),
        seed_policy: "default".to_string(),
        runner: "docker".to_string(),
        platform: "linux".to_string(),
        cpu: "x86_64".to_string(),
        threads: 4,
        io_mode: "local".to_string(),
    }
}

#[test]
fn summary_is_deterministic_across_ordering() -> anyhow::Result<()> {
    let suite = BenchmarkSuiteSpec::v1(
        "suite".to_string(),
        vec![DatasetSpec {
            id: "dataset".to_string(),
            hash: "hash".to_string(),
            size: 1,
            origin: "test".to_string(),
            class_label: "trueseq".to_string(),
            read_layout: "paired".to_string(),
        }],
        vec!["stage".to_string()],
        vec!["tool".to_string()],
        vec!["params".to_string()],
        ReplicatePolicy {
            count: 1,
            warmup: 0,
            seeds: vec![1],
        },
        DiversityRequirements {
            min_dataset_count: 1,
            min_classes: 1,
            min_read_layouts: 1,
        },
        vec![StratificationRequirement {
            key: "dataset_class".to_string(),
            required_values: vec!["trueseq".to_string()],
        }],
        AnalysisRequirements {
            require_bootstrap: false,
            require_outlier_detection: false,
            min_replicates_for_bootstrap: 5,
        },
    );

    let a = vec![
        obs("run-2", "dataset", "stage", "tool-b", "p2"),
        obs("run-1", "dataset", "stage", "tool-a", "p1"),
    ];
    let mut b = a.clone();
    b.reverse();

    let summary_a = summarize(&suite, &a, &BenchRunOptions::default())?;
    let summary_b = summarize(&suite, &b, &BenchRunOptions::default())?;
    assert_eq!(
        serde_json::to_string(&summary_a)?,
        serde_json::to_string(&summary_b)?
    );
    Ok(())
}
