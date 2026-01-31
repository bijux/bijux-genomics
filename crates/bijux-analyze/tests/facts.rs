use std::fs;
use std::path::PathBuf;

use bijux_analyze::facts_export::{summarize_facts, write_run_summary_json};
use bijux_analyze::load::load_facts;
use bijux_core::FactsRowV1;
use tempfile::TempDir;

#[test]
fn facts_loader_and_summary_work() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.path().join("facts.jsonl");
    let row = FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "ph".to_string(),
        input_hash: "ih".to_string(),
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
    };
    let payload = serde_json::to_string(&row)?;
    fs::write(&path, format!("{payload}\n"))?;

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
    assert_eq!(
        summary_json["stage_rows"][0]["bank_hashes"]["adapters"],
        "hash"
    );

    Ok(())
}

#[test]
fn run_summary_snapshot_is_stable() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let summary_path = dir.path().join("run_summary.json");
    let rows = vec![
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-2".to_string(),
            stage_id: "fastq.trim".to_string(),
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
            stage_id: "fastq.validate_pre".to_string(),
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
        .join("run_summary.json");
    let snapshot_raw = fs::read_to_string(&snapshot_path)?;
    let snapshot_value: serde_json::Value = serde_json::from_str(&snapshot_raw)?;
    assert_eq!(summary_value, snapshot_value);
    Ok(())
}
