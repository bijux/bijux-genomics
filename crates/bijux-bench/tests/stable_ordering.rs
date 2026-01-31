use std::collections::BTreeMap;

use bijux_bench::{BenchmarkObservation, MetricsEnvelope};
use bijux_bench::{summarize, BenchRunOptions, BenchmarkSuiteSpec};
use bijux_bench::{DatasetSpec, ReplicatePolicy};

fn obs(run_id: &str, dataset_id: &str, stage_id: &str, tool_id: &str, params_hash: &str) -> BenchmarkObservation {
    BenchmarkObservation {
        schema_version: "bijux.bench.observation.v1".to_string(),
        run_id: run_id.to_string(),
        dataset_id: dataset_id.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: "1.0".to_string(),
        image_digest: "sha256:abc".to_string(),
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
        }],
        vec!["stage".to_string()],
        vec!["tool".to_string()],
        vec!["params".to_string()],
        ReplicatePolicy {
            count: 1,
            warmup: 0,
            seeds: vec![1],
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
