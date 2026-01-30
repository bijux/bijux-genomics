use std::fs;

use bijux_core::run_index::{
    insert_run, insert_stage_row, list_runs, query_latest_runs, query_run, RunIndexEntry,
    StageIndexRow,
};
use tempfile::TempDir;

#[test]
fn run_index_insert_and_query() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let index_path = dir.path().join("index.jsonl");

    let run = RunIndexEntry {
        run_id: "run-1".to_string(),
        domain: "fastq".to_string(),
        pipeline: "fastq.trim".to_string(),
        stages: vec!["fastq.trim".to_string()],
        tools: vec!["fastp".to_string()],
        objective: None,
        platform: "local".to_string(),
        success: true,
    };
    insert_run(&index_path, &run)?;

    let row = StageIndexRow {
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        params_hash: "hash".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        artifacts: serde_json::json!({"plan": "plan.json"}),
    };
    insert_stage_row(&index_path, &row)?;

    let runs = list_runs(&index_path)?;
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run_id, "run-1");

    let latest = query_latest_runs(&index_path, 1)?;
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].run_id, "run-1");

    let found = query_run(&index_path, "run-1")?;
    assert!(found.is_some());

    let contents = fs::read_to_string(&index_path)?;
    assert!(contents.contains("\"stage_id\""));

    Ok(())
}
