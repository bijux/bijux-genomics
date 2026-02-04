use std::fs;

use bijux_core::run_index::{
    insert_run, insert_stage_row, list_runs, query_latest_runs, query_run, query_stage_rows,
    RunIndexEntry, StageIndexRow,
};

#[test]
fn run_index_insert_and_query() -> anyhow::Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
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

    let stage_rows = query_stage_rows(&index_path, Some("fastq.trim"), Some("fastp"))?;
    assert_eq!(stage_rows.len(), 1);

    let contents = fs::read_to_string(&index_path)?;
    assert!(contents.contains("\"stage_id\""));

    Ok(())
}

#[test]
fn run_index_latest_run_is_deterministic() -> anyhow::Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let index_path = dir.path().join("index.jsonl");

    for run_id in ["run-2", "run-1", "run-3"] {
        insert_run(
            &index_path,
            &RunIndexEntry {
                run_id: run_id.to_string(),
                domain: "fastq".to_string(),
                pipeline: "fastq.trim".to_string(),
                stages: vec!["fastq.trim".to_string()],
                tools: vec!["fastp".to_string()],
                objective: None,
                platform: "local".to_string(),
                success: true,
            },
        )?;
    }

    let latest = query_latest_runs(&index_path, 2)?;
    assert_eq!(latest.len(), 2);
    assert_eq!(latest[0].run_id, "run-2");
    assert_eq!(latest[1].run_id, "run-3");
    Ok(())
}

#[test]
fn run_index_query_by_stage_and_tool() -> anyhow::Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let index_path = dir.path().join("index.jsonl");

    insert_stage_row(
        &index_path,
        &StageIndexRow {
            run_id: "run-1".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            params_hash: "hash".to_string(),
            input_hash: "input".to_string(),
            output_hashes: vec!["out".to_string()],
            artifacts: serde_json::json!({}),
        },
    )?;
    insert_stage_row(
        &index_path,
        &StageIndexRow {
            run_id: "run-2".to_string(),
            stage_id: "fastq.validate_pre".to_string(),
            tool_id: "fastqvalidator".to_string(),
            params_hash: "hash2".to_string(),
            input_hash: "input2".to_string(),
            output_hashes: vec!["out2".to_string()],
            artifacts: serde_json::json!({}),
        },
    )?;

    let rows = query_stage_rows(&index_path, Some("fastq.trim"), Some("fastp"))?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].run_id, "run-1");
    Ok(())
}
