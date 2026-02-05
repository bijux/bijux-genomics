use anyhow::{anyhow, Result};
use bijux_runtime::*;

use bijux_analyze::aggregate::{
    metric_kind_for_stage, metric_spec, stage_metric_spec, BenchmarkContext, BenchmarkRecord,
    FastqTrimMetrics,
};
use bijux_analyze::report::{bench_schema_json, rank_trim_tools, write_run_summary_from_facts};
use bijux_core::metrics::MetricSet;
use bijux_core::primitives::measure::ExecutionMetrics;

#[test]
fn bench_schema_table_has_metrics() -> Result<()> {
    let json = bench_schema_json("fastq.trim")?;
    assert_eq!(json["stage"], "fastq.trim");
    assert!(!json["metrics"].as_array().unwrap_or(&Vec::new()).is_empty());
    Ok(())
}

#[test]
fn bench_schema_table_ordering_matches_registry() -> Result<()> {
    let json = bench_schema_json("fastq.trim")?;
    let observed: Vec<_> = json["metrics"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|entry| entry["name"].as_str())
        .map(ToString::to_string)
        .collect();
    let kind = metric_kind_for_stage("fastq.trim").ok_or_else(|| anyhow!("stage kind"))?;
    let spec = stage_metric_spec(kind);
    let expected: Vec<_> = spec
        .metrics
        .iter()
        .map(|metric_id| metric_spec(*metric_id).name.to_string())
        .collect();
    assert_eq!(observed, expected);
    Ok(())
}

#[test]
fn bench_schema_table_omits_range_when_missing() -> Result<()> {
    let json = bench_schema_json("fastq.trim")?;
    let empty = Vec::new();
    let entry = json["metrics"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .find(|metric| metric["name"] == "delta_metrics")
        .ok_or_else(|| anyhow!("delta_metrics"))?;
    assert!(entry.get("range").is_some());
    assert!(entry["range"].is_null());
    Ok(())
}

#[test]
fn run_summary_aggregation_works() -> Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let rows = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "ph".to_string(),
        input_hash: "ih".to_string(),
        output_hashes: vec!["oh".to_string()],
        runtime_s: 1.0,
        memory_mb: 32.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(10),
        reads_out: Some(9),
        bases_in: Some(100),
        bases_out: Some(90),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }];
    let summary_path = dir.path().join("run_summary.json");
    write_run_summary_from_facts(&summary_path, &rows)?;
    let summary_raw = std::fs::read_to_string(summary_path)?;
    let summary_value: serde_json::Value = serde_json::from_str(&summary_raw)?;
    assert_eq!(summary_value["runs"], 1);
    assert_eq!(summary_value["stages"], 1);
    Ok(())
}

#[test]
fn ranking_explanation_generation_has_modes() -> Result<()> {
    let metrics = FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: bijux_analyze::FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    };
    let record = BenchmarkRecord {
        context: BenchmarkContext {
            tool: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            runner: "docker".to_string(),
            platform: "linux".to_string(),
            input_hash: "ih".to_string(),
            parameters: bijux_analyze::model::JsonBlob::default(),
        },
        execution: ExecutionMetrics {
            runtime_s: 1.0,
            memory_mb: 10.0,
            exit_code: 0,
        },
        metrics: MetricSet {
            metrics_schema: "fastq_trim_v2".to_string(),
            version: 2,
            metrics,
        },
    };
    let rankings = rank_trim_tools(&[record])?;
    assert!(rankings.contains_key("FastestAcceptable"));
    assert!(rankings.contains_key("MostConservative"));
    assert!(rankings.contains_key("BalancedPareto"));
    Ok(())
}
