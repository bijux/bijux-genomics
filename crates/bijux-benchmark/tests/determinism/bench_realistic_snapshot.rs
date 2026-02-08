/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::BTreeMap;

use bijux_benchmark::{
    summarize, AnalysisRequirements, BenchRunOptions, BenchmarkObservation, BenchmarkSuiteSpec,
    DatasetSpec, DiversityRequirements, MetricsEnvelope, ReplicatePolicy,
    StratificationRequirement,
};

fn observation(
    run_id: &str,
    dataset_id: &str,
    dataset_class: &str,
    read_layout: &str,
    tool_id: &str,
    params_hash: &str,
    runtime_s: f64,
) -> BenchmarkObservation {
    BenchmarkObservation {
        schema_version: "bijux.bench.observation.v1".to_string(),
        run_id: run_id.to_string(),
        dataset_id: dataset_id.to_string(),
        dataset_class: dataset_class.to_string(),
        read_layout: read_layout.to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: tool_id.to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: "sha256:abc".to_string(),
        container_digest: "sha256:abc".to_string(),
        params_hash: params_hash.to_string(),
        input_hash: "input".to_string(),
        runtime_s,
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
    }
}

#[test]
fn realistic_suite_snapshot() -> anyhow::Result<()> {
    let suite = BenchmarkSuiteSpec::v1(
        "suite-elite".to_string(),
        vec![
            DatasetSpec {
                id: "ds-1".to_string(),
                hash: "hash-1".to_string(),
                size: 100,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            },
            DatasetSpec {
                id: "ds-2".to_string(),
                hash: "hash-2".to_string(),
                size: 200,
                origin: "synthetic".to_string(),
                class_label: "nextera".to_string(),
                read_layout: "paired".to_string(),
            },
        ],
        vec!["fastq.trim".to_string()],
        vec!["fastp".to_string(), "cutadapt".to_string()],
        vec!["params-a".to_string()],
        ReplicatePolicy {
            count: 3,
            warmup: 0,
            seeds: vec![1, 2, 3],
        },
        DiversityRequirements {
            min_dataset_count: 2,
            min_classes: 2,
            min_read_layouts: 1,
        },
        vec![StratificationRequirement {
            key: "dataset_class".to_string(),
            required_values: vec!["trueseq".to_string(), "nextera".to_string()],
        }],
        AnalysisRequirements {
            require_bootstrap: false,
            require_outlier_detection: true,
            min_replicates_for_bootstrap: 5,
        },
    );

    let mut observations = Vec::new();
    for (dataset_id, dataset_class) in [("ds-1", "trueseq"), ("ds-2", "nextera")] {
        observations.push(observation(
            "run-1",
            dataset_id,
            dataset_class,
            "paired",
            "fastp",
            "params-a",
            1.0,
        ));
        observations.push(observation(
            "run-2",
            dataset_id,
            dataset_class,
            "paired",
            "fastp",
            "params-a",
            1.1,
        ));
        observations.push(observation(
            "run-3",
            dataset_id,
            dataset_class,
            "paired",
            "fastp",
            "params-a",
            0.9,
        ));
        observations.push(observation(
            "run-4",
            dataset_id,
            dataset_class,
            "paired",
            "cutadapt",
            "params-a",
            1.4,
        ));
        observations.push(observation(
            "run-5",
            dataset_id,
            dataset_class,
            "paired",
            "cutadapt",
            "params-a",
            1.5,
        ));
        observations.push(observation(
            "run-6",
            dataset_id,
            dataset_class,
            "paired",
            "cutadapt",
            "params-a",
            1.3,
        ));
    }

    let summary = summarize(&suite, &observations, &BenchRunOptions::default())?;
    let snapshot = serde_json::to_value(&summary).expect("serialize summary");
    insta::assert_json_snapshot!(bijux_testkit::snapshot_normalize_json(&snapshot), @r#"
    {
      "invalid_reasons": [],
      "rows": [
        {
          "completeness": 1.0,
          "dataset_class": "trueseq",
          "dataset_id": "ds-1",
          "failure_rate": 0.0,
          "low_power": false,
          "memory": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "memory_mb",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.0,
              "median": 100.0,
              "n": 3,
              "trimmed_mean": 100.0
            }
          },
          "metrics": [],
          "n_effective": 3,
          "params_hash": "params-a",
          "read_layout": "paired",
          "runtime": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "runtime_s",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.09999999999999987,
              "median": 1.4,
              "n": 3,
              "trimmed_mean": 1.4000000000000001
            }
          },
          "stage_id": "fastq.trim",
          "tool_id": "cutadapt"
        },
        {
          "completeness": 1.0,
          "dataset_class": "trueseq",
          "dataset_id": "ds-1",
          "failure_rate": 0.0,
          "low_power": false,
          "memory": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "memory_mb",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.0,
              "median": 100.0,
              "n": 3,
              "trimmed_mean": 100.0
            }
          },
          "metrics": [],
          "n_effective": 3,
          "params_hash": "params-a",
          "read_layout": "paired",
          "runtime": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "runtime_s",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.09999999999999998,
              "median": 1.0,
              "n": 3,
              "trimmed_mean": 1.0
            }
          },
          "stage_id": "fastq.trim",
          "tool_id": "fastp"
        },
        {
          "completeness": 1.0,
          "dataset_class": "nextera",
          "dataset_id": "ds-2",
          "failure_rate": 0.0,
          "low_power": false,
          "memory": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "memory_mb",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.0,
              "median": 100.0,
              "n": 3,
              "trimmed_mean": 100.0
            }
          },
          "metrics": [],
          "n_effective": 3,
          "params_hash": "params-a",
          "read_layout": "paired",
          "runtime": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "runtime_s",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.09999999999999987,
              "median": 1.4,
              "n": 3,
              "trimmed_mean": 1.4000000000000001
            }
          },
          "stage_id": "fastq.trim",
          "tool_id": "cutadapt"
        },
        {
          "completeness": 1.0,
          "dataset_class": "nextera",
          "dataset_id": "ds-2",
          "failure_rate": 0.0,
          "low_power": false,
          "memory": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "memory_mb",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.0,
              "median": 100.0,
              "n": 3,
              "trimmed_mean": 100.0
            }
          },
          "metrics": [],
          "n_effective": 3,
          "params_hash": "params-a",
          "read_layout": "paired",
          "runtime": {
            "ci_high": null,
            "ci_low": null,
            "metric_id": "runtime_s",
            "n": 3,
            "outlier_count": 0,
            "outlier_replicates": [],
            "power_warning": true,
            "practical_threshold": 0.05,
            "stats": {
              "iqr": 0.0,
              "mad": 0.09999999999999998,
              "median": 1.0,
              "n": 3,
              "trimmed_mean": 1.0
            }
          },
          "stage_id": "fastq.trim",
          "tool_id": "fastp"
        }
      ],
      "schema_version": "bijux.bench.summary.v1",
      "scientifically_invalid": false,
      "strata": [
        {
          "dataset_class": "nextera",
          "low_power_count": 0,
          "row_count": 2,
          "stage_id": "fastq.trim"
        },
        {
          "dataset_class": "trueseq",
          "low_power_count": 0,
          "row_count": 2,
          "stage_id": "fastq.trim"
        }
      ],
      "suite_id": "suite-elite",
      "warnings": []
    }
    "#);
    Ok(())
}
