use bijux_dna_analyze::model::FactTable;
use bijux_dna_runtime::*;

#[test]
fn fact_table_rejects_missing_ids() {
    let row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run".to_string(),
        stage_id: String::new(),
        tool_id: String::new(),
        tool_version: "1.0".to_string(),
        image_digest: None,
        trace_id: "t".to_string(),
        span_id: "s".to_string(),
        params_hash: "p".to_string(),
        input_hash: "i".to_string(),
        output_hashes: vec![],
        runtime_s: 0.0,
        memory_mb: 0.0,
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
    assert!(FactTable::from_facts(&[row]).is_err());
}

#[test]
fn fact_table_rejects_partial_deltas() {
    let row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "1.0".to_string(),
        image_digest: None,
        trace_id: "t".to_string(),
        span_id: "s".to_string(),
        params_hash: "p".to_string(),
        input_hash: "i".to_string(),
        output_hashes: vec![],
        runtime_s: 1.0,
        memory_mb: 1.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(100),
        reads_out: None,
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    };
    assert!(FactTable::from_facts(&[row]).is_err());
}

#[test]
fn fact_table_rejects_unknown_stage() {
    let row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run".to_string(),
        stage_id: "fastq.unknown".to_string(),
        tool_id: "tool".to_string(),
        tool_version: "1.0".to_string(),
        image_digest: None,
        trace_id: "t".to_string(),
        span_id: "s".to_string(),
        params_hash: "p".to_string(),
        input_hash: "i".to_string(),
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
    assert!(FactTable::from_facts(&[row]).is_err());
}
