use bijux_benchmark::{
    AnalysisRequirements, BenchmarkSuiteSpec, BenchmarkSummary, DatasetSpec, DiversityRequirements,
    ReplicatePolicy, StratificationRequirement,
};

#[test]
fn bench_contract_snapshot() -> anyhow::Result<()> {
    let spec = BenchmarkSuiteSpec::v1(
        "suite-1".to_string(),
        vec![DatasetSpec {
            id: "dataset-1".to_string(),
            hash: "hash-1".to_string(),
            size: 100,
            origin: "synthetic".to_string(),
            class_label: "trueseq".to_string(),
            read_layout: "paired".to_string(),
        }],
        vec!["fastq.trim".to_string()],
        vec!["tool-a".to_string()],
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
    let summary = BenchmarkSummary::v1("suite-1".to_string(), Vec::new(), Vec::new(), Vec::new());
    let payload = serde_json::json!({
        "spec": spec,
        "summary": summary,
    });
    insta::assert_json_snapshot!(payload, @r###"
    {
      "spec": {
        "analysis_requirements": {
          "min_replicates_for_bootstrap": 5,
          "require_bootstrap": false,
          "require_outlier_detection": true
        },
        "datasets": [
          {
            "class_label": "trueseq",
            "hash": "hash-1",
            "id": "dataset-1",
            "origin": "synthetic",
            "read_layout": "paired",
            "size": 100
          }
        ],
        "diversity": {
          "min_classes": 1,
          "min_dataset_count": 1,
          "min_read_layouts": 1
        },
        "params": [
          "params-a"
        ],
        "replicate_policy": {
          "count": 3,
          "seeds": [
            1,
            2,
            3
          ],
          "warmup": 0
        },
        "schema_version": "bijux.bench.suite.v1",
        "stages": [
          "fastq.trim"
        ],
        "stratifications": [
          {
            "key": "dataset_class",
            "required_values": [
              "trueseq"
            ]
          }
        ],
        "suite_id": "suite-1",
        "tools": [
          "tool-a"
        ]
      },
      "summary": {
        "invalid_reasons": [],
        "rows": [],
        "schema_version": "bijux.bench.summary.v1",
        "scientifically_invalid": false,
        "strata": [],
        "suite_id": "suite-1",
        "warnings": []
      }
    }
    "###);
    Ok(())
}

#[test]
fn suite_requires_stratification_metadata() {
    let suite = BenchmarkSuiteSpec::v1(
        "suite-missing-strata".to_string(),
        vec![DatasetSpec {
            id: "dataset-1".to_string(),
            hash: "hash-1".to_string(),
            size: 100,
            origin: "synthetic".to_string(),
            class_label: "trueseq".to_string(),
            read_layout: "paired".to_string(),
        }],
        vec!["fastq.trim".to_string()],
        vec!["tool-a".to_string()],
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
            required_values: vec!["nextera".to_string()],
        }],
        AnalysisRequirements {
            require_bootstrap: false,
            require_outlier_detection: false,
            min_replicates_for_bootstrap: 5,
        },
    );
    let result =
        bijux_benchmark::summarize(&suite, &[], &bijux_benchmark::BenchRunOptions::default());
    assert!(
        result.is_err(),
        "suite validation should fail on missing strata"
    );
}
