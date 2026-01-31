use bijux_analyze::export::write_dashboard_facts_jsonl;
use bijux_core::FactsRowV1;

#[test]
fn dashboard_contract_has_required_fields_and_sorting() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    let path = dir.path().join("dashboard.jsonl");
    let rows = vec![
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-b".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: Some("sha256:abc".to_string()),
            trace_id: "trace-b".to_string(),
            span_id: "span-b".to_string(),
            params_hash: "ph-b".to_string(),
            input_hash: "ih-b".to_string(),
            output_hashes: vec![],
            runtime_s: 1.2,
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
        },
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-a".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: Some("sha256:def".to_string()),
            trace_id: "trace-a".to_string(),
            span_id: "span-a".to_string(),
            params_hash: "ph-a".to_string(),
            input_hash: "ih-a".to_string(),
            output_hashes: vec![],
            runtime_s: 1.0,
            memory_mb: 9.0,
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
        },
    ];
    write_dashboard_facts_jsonl(&path, &rows)?;
    let raw = std::fs::read_to_string(&path)?;
    let mut lines = raw.lines();
    let first_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("dashboard facts jsonl missing first row"))?;
    let first: serde_json::Value = serde_json::from_str(first_line)?;
    let required = [
        "schema_version",
        "run_id",
        "stage_id",
        "tool_id",
        "tool_version",
        "params_hash",
        "input_hash",
        "runtime_s",
        "memory_mb",
        "exit_code",
        "bank_hashes",
        "metrics",
        "reports",
        "artifacts",
        "trace_id",
        "span_id",
    ];
    for key in &required {
        assert!(first.get(key).is_some(), "missing field {key}");
    }
    assert_eq!(first["run_id"], "run-a");
    Ok(())
}
