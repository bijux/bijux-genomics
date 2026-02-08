use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_benchmark::{
    compare, summarize, AnalysisRequirements, BenchRunOptions, BenchmarkObservation,
    BenchmarkSuiteSpec, DatasetSpec, DiversityRequirements, MetricsEnvelope, ReplicatePolicy,
    StratificationRequirement,
};

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-benchmark__{group}__{name}")
}

#[test]
fn bench_compare_snapshot() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let suite_a = BenchmarkSuiteSpec::v1(
        "suite-a".to_string(),
        vec![DatasetSpec {
            id: "dataset-1".to_string(),
            hash: "hash-1".to_string(),
            size: 100,
            origin: "synthetic".to_string(),
            class_label: "trueseq".to_string(),
            read_layout: "paired".to_string(),
        }],
        vec!["fastq.trim".to_string()],
        vec!["fastp".to_string()],
        vec!["params-a".to_string()],
        ReplicatePolicy {
            count: 3,
            warmup: 0,
            seeds: vec![1, 2, 3],
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
            require_outlier_detection: true,
            min_replicates_for_bootstrap: 5,
        },
    );
    let suite_b = BenchmarkSuiteSpec::v1(
        "suite-b".to_string(),
        vec![DatasetSpec {
            id: "dataset-1".to_string(),
            hash: "hash-1".to_string(),
            size: 100,
            origin: "synthetic".to_string(),
            class_label: "trueseq".to_string(),
            read_layout: "paired".to_string(),
        }],
        vec!["fastq.trim".to_string()],
        vec!["fastp".to_string()],
        vec!["params-a".to_string()],
        ReplicatePolicy {
            count: 3,
            warmup: 0,
            seeds: vec![1, 2, 3],
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
            require_outlier_detection: true,
            min_replicates_for_bootstrap: 5,
        },
    );

    let obs_a = BenchmarkObservation {
        schema_version: "bijux.bench.observation.v1".to_string(),
        run_id: "run-a".to_string(),
        dataset_id: "dataset-1".to_string(),
        dataset_class: "trueseq".to_string(),
        read_layout: "paired".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: "sha256:abc".to_string(),
        container_digest: "sha256:abc".to_string(),
        params_hash: "params-a".to_string(),
        input_hash: "input".to_string(),
        runtime_s: 1.0,
        memory_mb: 100.0,
        exit_code: 0,
        failure_kind: None,
        metrics: MetricsEnvelope {
            stage_id: "fastq.trim".to_string(),
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
    };
    let obs_b = BenchmarkObservation {
        runtime_s: 2.0,
        memory_mb: 120.0,
        ..obs_a.clone()
    };

    let summary_a = summarize(
        &suite_a,
        &[obs_a.clone(), obs_a.clone(), obs_a],
        &BenchRunOptions::default(),
    )?;
    let summary_b = summarize(
        &suite_b,
        &[obs_b.clone(), obs_b.clone(), obs_b],
        &BenchRunOptions::default(),
    )?;
    let comparison = compare(&summary_a, &summary_b)?;
    let rendered = serde_json::to_string_pretty(&comparison)?;
    let snapshot_file = format!("{}.json", snapshot_name("contracts", "bench_compare"));
    let snapshot_path = manifest_dir.join("tests").join("snapshots").join(snapshot_file);
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
