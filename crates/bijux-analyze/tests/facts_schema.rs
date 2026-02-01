use anyhow::Result;
use bijux_core::FactsRowV1;

#[test]
fn facts_schema_contract_has_required_fields() -> Result<()> {
    let row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "params".to_string(),
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
    };
    let value = serde_json::to_value(&row)?;
    let required = [
        "run_id",
        "stage_id",
        "tool_id",
        "params_hash",
        "bank_hashes",
        "runtime_s",
        "reads_in",
        "reads_out",
        "bases_in",
        "bases_out",
    ];
    for key in required {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    assert_eq!(
        value.get("schema_version").and_then(|v| v.as_str()),
        Some("bijux.facts.v1")
    );
    Ok(())
}

#[test]
fn facts_schema_allows_unknown_fields() -> Result<()> {
    let row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-unknown".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: None,
        trace_id: "trace".to_string(),
        span_id: "span".to_string(),
        params_hash: "params".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec![],
        runtime_s: 1.0,
        memory_mb: 1.0,
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
    let mut value = serde_json::to_value(&row)?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert("future_field".to_string(), serde_json::json!("ok"));
    }
    let _: FactsRowV1 = serde_json::from_value(value)?;
    Ok(())
}
