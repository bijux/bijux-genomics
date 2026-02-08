use bijux_runtime::*;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_analyze::decision::compare::{compare_robust_stats, trace_for_robust_stats};
use bijux_analyze::decision::score::{decision_trace_for_input, RankInput, RankingMode};

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-analyze__{group}__{name}")
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(name)
}

#[test]
fn decision_trace_missing_metrics_snapshot() -> Result<()> {
    let input = RankInput {
        tool: "missing".to_string(),
        runtime_s: 1.0,
        memory_mb: 10.0,
        read_retention: None,
        base_retention: None,
        error_reduction_proxy: None,
    };
    let trace = decision_trace_for_input(RankingMode::BalancedPareto, &input);
    let rendered = serde_json::to_string_pretty(&trace)?;
    let snapshot = fs::read_to_string(snapshot_path(&format!(
        "{}.json",
        snapshot_name("semantics", "decision_trace_missing")
    )))?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn decision_trace_outliers_snapshot() -> Result<()> {
    let base = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "tool".to_string(),
        tool_version: "0.1".to_string(),
        image_digest: None,
        trace_id: "t1".to_string(),
        span_id: "s1".to_string(),
        params_hash: "p1".to_string(),
        input_hash: "i1".to_string(),
        output_hashes: Vec::new(),
        runtime_s: 1.0,
        memory_mb: 10.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: None,
        reads_out: None,
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    };
    let mut rows = vec![];
    for _ in 0..5 {
        rows.push(base.clone());
    }
    rows.push(FactsRowV1 {
        runtime_s: 1000.0,
        memory_mb: 2000.0,
        ..base
    });
    let stats = compare_robust_stats(&rows)?;
    let trace = trace_for_robust_stats(&stats);
    let rendered = serde_json::to_string_pretty(&trace)?;
    let snapshot = fs::read_to_string(snapshot_path(&format!(
        "{}.json",
        snapshot_name("semantics", "decision_trace_outliers")
    )))?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
