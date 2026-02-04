use bijux_analyze::report::write_run_report_from_facts;
use bijux_bench::{
    summarize, BenchRunOptions, BenchmarkObservation, BenchmarkSuiteSpec, DatasetSpec,
    MetricsEnvelope, ReplicatePolicy,
};
use bijux_core::FactsRowV1;
use std::collections::BTreeMap;

#[test]
fn analyze_consumes_bench_summary() -> anyhow::Result<()> {
    let temp = bijux_infra::temp_dir("bijux")?;
    let base_dir = temp.path();
    let bench_dir = base_dir.join("bench");
    bijux_infra::ensure_dir(&bench_dir)?;

    let suite = BenchmarkSuiteSpec::v1(
        "suite-1".to_string(),
        vec![DatasetSpec {
            id: "dataset-1".to_string(),
            hash: "hash-1".to_string(),
            size: 100,
            origin: "synthetic".to_string(),
        }],
        vec!["fastq.trim".to_string()],
        vec!["fastp".to_string()],
        vec!["params-a".to_string()],
        ReplicatePolicy {
            count: 3,
            warmup: 0,
            seeds: vec![1, 2, 3],
        },
    );
    let obs = BenchmarkObservation {
        schema_version: "bijux.bench.observation.v1".to_string(),
        run_id: "run-1".to_string(),
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
    let summary = summarize(
        &suite,
        &[obs.clone(), obs.clone(), obs],
        &BenchRunOptions::default(),
    )?;
    let summary_path = bench_dir.join("summary.json");
    bijux_infra::write_bytes(&summary_path, serde_json::to_vec_pretty(&summary)?)?;

    let facts = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace".to_string(),
        span_id: "span".to_string(),
        params_hash: "params-a".to_string(),
        input_hash: "input".to_string(),
        output_hashes: Vec::new(),
        runtime_s: 1.0,
        memory_mb: 100.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(100),
        reads_out: Some(90),
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }];

    let defaults = serde_json::json!({
        "pipeline_id": "fastq-to-fastq__default__v1",
        "tools": {},
        "params": {},
        "thresholds": {},
        "tool_provenance": {},
        "param_provenance": {},
        "assumptions": [],
        "citations": {},
    });
    bijux_infra::write_bytes(
        base_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;

    let report_path = write_run_report_from_facts(base_dir, &facts)?;
    let report_raw = std::fs::read_to_string(report_path)?;
    let report_json: serde_json::Value = serde_json::from_str(&report_raw)?;
    let bench_section = report_json
        .get("sections")
        .and_then(|sections| sections.get("bench_summary"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    assert_eq!(
        bench_section.get("suite_id").and_then(|v| v.as_str()),
        Some("suite-1")
    );
    Ok(())
}
