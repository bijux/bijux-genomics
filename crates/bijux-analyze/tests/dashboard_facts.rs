use std::fs;
use std::path::PathBuf;

use bijux_analyze::export::write_dashboard_facts_jsonl;
use bijux_core::FactsRowV1;

fn facts_row(run_id: &str, stage_id: &str, tool_id: &str, params: &str) -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: run_id.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: "1.0".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace".to_string(),
        span_id: "span".to_string(),
        params_hash: params.to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        runtime_s: 1.0,
        memory_mb: 10.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({"adapters": "hash"}),
        reads_in: Some(10),
        reads_out: Some(9),
        bases_in: Some(100),
        bases_out: Some(90),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }
}

#[test]
fn dashboard_facts_snapshot_is_stable() -> anyhow::Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    let rows = vec![
        facts_row("run-2", "fastq.trim", "fastp", "b"),
        facts_row("run-1", "fastq.validate_pre", "fastqvalidator", "a"),
    ];
    write_dashboard_facts_jsonl(&path, &rows)?;
    let rendered = fs::read_to_string(&path)?;
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("dashboard_facts.jsonl");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered, snapshot);
    Ok(())
}
