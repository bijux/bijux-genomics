use bijux_bench::contract::{BenchmarkDecision, BenchmarkSummary};
use bijux_bench::suite::BenchmarkSuiteSpec;

#[test]
fn bench_contract_snapshot() -> anyhow::Result<()> {
    let spec = BenchmarkSuiteSpec::v1(
        "suite-1".to_string(),
        vec!["dataset-1".to_string()],
        vec!["fastq.trim".to_string()],
        vec!["tool-a".to_string()],
        vec!["params-a".to_string()],
        3,
    );
    let summary = BenchmarkSummary::v1(
        "suite-1".to_string(),
        "dataset-1".to_string(),
        2,
        vec!["low_sample_size".to_string()],
        vec![BenchmarkDecision {
            tool: "tool-a".to_string(),
            passes: true,
            missing_metrics: vec![],
            rationale: vec!["metric:runtime:1.0 threshold:2.0".to_string()],
        }],
    );
    let payload = serde_json::json!({
        "spec": spec,
        "summary": summary,
    });
    insta::assert_json_snapshot!(payload, @r###"
{
  "spec": {
    "datasets": [
      "dataset-1"
    ],
    "params": [
      "params-a"
    ],
    "replicates": 3,
    "schema_version": "bijux.bench.suite.v1",
    "stages": [
      "fastq.trim"
    ],
    "suite_id": "suite-1",
    "tools": [
      "tool-a"
    ]
  },
  "summary": {
    "dataset_hash": "dataset-1",
    "decisions": [
      {
        "missing_metrics": [],
        "passes": true,
        "rationale": [
          "metric:runtime:1.0 threshold:2.0"
        ],
        "tool": "tool-a"
      }
    ],
    "observations": 2,
    "schema_version": "bijux.bench.summary.v1",
    "suite_id": "suite-1",
    "warnings": [
      "low_sample_size"
    ]
  }
}
"###);
    Ok(())
}
