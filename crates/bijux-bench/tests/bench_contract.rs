use bijux_bench::{BenchmarkSuiteSpec, BenchmarkSummary, DatasetSpec, ReplicatePolicy};

#[test]
fn bench_contract_snapshot() -> anyhow::Result<()> {
    let spec = BenchmarkSuiteSpec::v1(
        "suite-1".to_string(),
        vec![DatasetSpec {
            id: "dataset-1".to_string(),
            hash: "hash-1".to_string(),
            size: 100,
            origin: "synthetic".to_string(),
        }],
        vec!["fastq.trim".to_string()],
        vec!["tool-a".to_string()],
        vec!["params-a".to_string()],
        ReplicatePolicy {
            count: 3,
            warmup: 0,
            seeds: vec![1, 2, 3],
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
        "datasets": [
          {
            "hash": "hash-1",
            "id": "dataset-1",
            "origin": "synthetic",
            "size": 100
          }
        ],
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
