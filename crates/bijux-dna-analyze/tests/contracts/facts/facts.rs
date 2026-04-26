use bijux_dna_runtime::*;
use std::fs;
use std::path::PathBuf;

use bijux_dna_analyze::exports::{
    summarize_facts, write_run_summary_json, write_stage_summary_csv,
};
use bijux_dna_analyze::load::load_facts;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-analyze__{group}__{name}")
}

fn facts_row(input_hash: &str) -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "ph".to_string(),
        input_hash: input_hash.to_string(),
        output_hashes: vec!["oh".to_string()],
        runtime_s: 1.5,
        memory_mb: 42.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({"adapters": "hash"}),
        reads_in: Some(10),
        reads_out: Some(9),
        bases_in: Some(100),
        bases_out: Some(90),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({
            "stage_report": "stage_report.json",
            "retention_report": "retention_report.json"
        }),
        artifacts: serde_json::json!({"metrics_envelope": "metrics.json"}),
    }
}

#[test]
fn facts_loader_and_summary_work() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    let row = facts_row("ih");
    let payload = serde_json::to_string(&row)?;
    bijux_dna_infra::write_bytes(&path, format!("{payload}\n"))?;

    let rows = load_facts(&path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    assert_eq!(rows.len(), 1);
    let summary = summarize_facts(&rows);
    assert_eq!(summary.runs, 1);
    assert_eq!(summary.stages, 1);
    assert!((summary.total_runtime_s - 1.5).abs() < 1e-6);

    let summary_path = dir.path().join("run_summary.json");
    write_run_summary_json(&summary_path, &rows)?;
    let summary_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(summary_path)?)?;
    assert_eq!(summary_json["schema_version"], "bijux.run_summary.v1");
    assert_eq!(summary_json["facts_path"], "facts.jsonl");
    assert_eq!(summary_json["report_path"], "report.json");
    assert_eq!(summary_json["telemetry_path"], "telemetry/events.jsonl");
    assert_eq!(summary_json["runs"], 1);
    assert_eq!(summary_json["stages"], 1);
    assert_eq!(summary_json["stage_rows"][0]["tool_version"], "0.23.4");
    assert_eq!(summary_json["stage_rows"][0]["image_digest"], "sha256:abc");
    assert_eq!(summary_json["stage_rows"][0]["bank_hashes"]["adapters"], "hash");

    Ok(())
}

#[test]
fn facts_loader_orders_full_identity_key() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    let row_b = facts_row("input-b");
    let row_a = facts_row("input-a");
    bijux_dna_infra::write_bytes(
        &path,
        format!("{}\n{}\n", serde_json::to_string(&row_b)?, serde_json::to_string(&row_a)?),
    )?;

    let rows = load_facts(&path).map_err(|err| anyhow::anyhow!(err.to_string()))?;

    assert_eq!(
        rows.iter().map(|row| row.input_hash.as_str()).collect::<Vec<_>>(),
        vec!["input-a", "input-b"]
    );
    Ok(())
}

#[test]
fn stage_summary_csv_quotes_carriage_returns() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("stage_summary.csv");
    let mut row = facts_row("ih");
    row.tool_version = "0.23\r4".to_string();

    write_stage_summary_csv(&path, &[row])?;
    let csv = fs::read_to_string(&path)?;

    assert!(csv.contains("\"0.23\r4\""));
    Ok(())
}

#[test]
fn run_summary_snapshot_is_stable() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let summary_path = dir.path().join("run_summary.json");
    let rows = vec![
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-2".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: Some("sha256:abc".to_string()),
            trace_id: "trace-2".to_string(),
            span_id: "span-2".to_string(),
            params_hash: "ph2".to_string(),
            input_hash: "ih2".to_string(),
            output_hashes: vec!["oh2".to_string()],
            runtime_s: 2.0,
            memory_mb: 43.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({"adapters": "hash2"}),
            reads_in: Some(20),
            reads_out: Some(18),
            bases_in: Some(200),
            bases_out: Some(180),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({"stage_report": "stage_report.json"}),
            artifacts: serde_json::json!({}),
        },
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-2".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "fastqvalidator".to_string(),
            tool_version: "1.0".to_string(),
            image_digest: Some("sha256:def".to_string()),
            trace_id: "trace-3".to_string(),
            span_id: "span-3".to_string(),
            params_hash: "ph3".to_string(),
            input_hash: "ih3".to_string(),
            output_hashes: vec!["oh3".to_string()],
            runtime_s: 1.0,
            memory_mb: 30.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({}),
            reads_in: Some(20),
            reads_out: Some(20),
            bases_in: Some(200),
            bases_out: Some(200),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({"stage_report": "stage_report.json"}),
            artifacts: serde_json::json!({}),
        },
    ];
    write_run_summary_json(&summary_path, &rows)?;
    let summary_raw = fs::read_to_string(&summary_path)?;
    let summary_value: serde_json::Value = serde_json::from_str(&summary_raw)?;
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(snapshot_name("schemas", "run_summary"))
        .with_extension("json");
    let snapshot_raw = fs::read_to_string(&snapshot_path)?;
    let snapshot_value: serde_json::Value = serde_json::from_str(&snapshot_raw)?;
    assert_eq!(summary_value, snapshot_value);
    Ok(())
}
