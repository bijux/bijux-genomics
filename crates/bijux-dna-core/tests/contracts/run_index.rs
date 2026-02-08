use std::fs;
use std::io::Write;

use bijux_dna_core::contract::{
    list_runs, query_latest_runs, query_run, query_stage_rows, ContractVersion, RunIndexEntry,
    RunIndexLine, StageIndexRow,
};
use bijux_dna_core::ids::{PipelineId, RunId, StageId, ToolId};

fn append_line(path: &std::path::Path, line: &str) -> anyhow::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn write_run(path: &std::path::Path, run: &RunIndexEntry) -> anyhow::Result<()> {
    let line = RunIndexLine {
        schema_version: 1,
        contract_version: ContractVersion::v1(),
        run: Some(run.clone()),
        stage: None,
    };
    let payload = serde_json::to_string(&line)?;
    append_line(path, &payload)
}

fn write_stage_row(path: &std::path::Path, row: &StageIndexRow) -> anyhow::Result<()> {
    let line = RunIndexLine {
        schema_version: 1,
        contract_version: ContractVersion::v1(),
        run: None,
        stage: Some(row.clone()),
    };
    let payload = serde_json::to_string(&line)?;
    append_line(path, &payload)
}

#[test]
fn run_index_insert_and_query() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let index_path = dir.path().join("index.jsonl");

    let run = RunIndexEntry {
        run_id: RunId("run-1".to_string()),
        domain: "fastq".to_string(),
        pipeline: PipelineId::new("fastq.trim"),
        stages: vec![StageId::new("fastq.trim")],
        tools: vec![ToolId::new("fastp")],
        objective: None,
        platform: "local".to_string(),
        success: true,
    };
    write_run(&index_path, &run)?;

    let row = StageIndexRow {
        run_id: RunId("run-1".to_string()),
        stage_id: StageId::new("fastq.trim"),
        tool_id: ToolId::new("fastp"),
        params_hash: "hash".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        artifacts: serde_json::json!({"plan": "plan.json"}),
    };
    write_stage_row(&index_path, &row)?;

    let runs = list_runs(&index_path)?;
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run_id.as_str(), "run-1");

    let latest = query_latest_runs(&index_path, 1)?;
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].run_id.as_str(), "run-1");

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
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let index_path = dir.path().join("index.jsonl");

    for run_id in ["run-2", "run-1", "run-3"] {
        write_run(
            &index_path,
            &RunIndexEntry {
                run_id: RunId(run_id.to_string()),
                domain: "fastq".to_string(),
                pipeline: PipelineId::new("fastq.trim"),
                stages: vec![StageId::new("fastq.trim")],
                tools: vec![ToolId::new("fastp")],
                objective: None,
                platform: "local".to_string(),
                success: true,
            },
        )?;
    }

    let latest = query_latest_runs(&index_path, 2)?;
    assert_eq!(latest.len(), 2);
    assert_eq!(latest[0].run_id.as_str(), "run-2");
    assert_eq!(latest[1].run_id.as_str(), "run-3");
    Ok(())
}

#[test]
fn run_index_query_by_stage_and_tool() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let index_path = dir.path().join("index.jsonl");

    write_stage_row(
        &index_path,
        &StageIndexRow {
            run_id: RunId("run-1".to_string()),
            stage_id: StageId::new("fastq.trim"),
            tool_id: ToolId::new("fastp"),
            params_hash: "hash".to_string(),
            input_hash: "input".to_string(),
            output_hashes: vec!["out".to_string()],
            artifacts: serde_json::json!({}),
        },
    )?;
    write_stage_row(
        &index_path,
        &StageIndexRow {
            run_id: RunId("run-2".to_string()),
            stage_id: StageId::new("fastq.validate_pre"),
            tool_id: ToolId::new("fastqvalidator"),
            params_hash: "hash2".to_string(),
            input_hash: "input2".to_string(),
            output_hashes: vec!["out2".to_string()],
            artifacts: serde_json::json!({}),
        },
    )?;

    let rows = query_stage_rows(&index_path, Some("fastq.trim"), Some("fastp"))?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].run_id.as_str(), "run-1");
    Ok(())
}
