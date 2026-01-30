use std::fs;

use bijux_analyze::facts::{load_facts_jsonl, summarize_facts, write_run_summary_json};
use bijux_core::FactsRowV1;
use tempfile::TempDir;

#[test]
fn facts_loader_and_summary_work() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.path().join("facts.jsonl");
    let row = FactsRowV1 {
        schema_version: "bijux.facts_row.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        params_hash: "ph".to_string(),
        input_hash: "ih".to_string(),
        output_hashes: vec!["oh".to_string()],
        runtime_s: 1.5,
        memory_mb: 42.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(10),
        reads_out: Some(9),
        bases_in: Some(100),
        bases_out: Some(90),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({"stage_report": "stage_report.json"}),
        artifacts: serde_json::json!({"metrics_envelope": "metrics.json"}),
    };
    let payload = serde_json::to_string(&row)?;
    fs::write(&path, format!("{payload}\n"))?;

    let rows = load_facts_jsonl(&path)?;
    assert_eq!(rows.len(), 1);
    let summary = summarize_facts(&rows);
    assert_eq!(summary.runs, 1);
    assert_eq!(summary.stages, 1);
    assert!((summary.total_runtime_s - 1.5).abs() < 1e-6);

    let summary_path = dir.path().join("run_summary.json");
    write_run_summary_json(&summary_path, &rows)?;
    let summary_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(summary_path)?)?;
    assert_eq!(summary_json["schema_version"], "bijux.run_summary.v1");
    assert_eq!(summary_json["runs"], 1);
    assert_eq!(summary_json["stages"], 1);

    Ok(())
}
