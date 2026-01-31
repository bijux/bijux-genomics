use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_analyze::{
    facts::load_facts_jsonl, report::write_run_report_from_facts,
    report::write_run_summary_from_facts,
};
use bijux_core::{FactsRowV1, ReportSchemaV1, RetentionReportV1, StageReportV1};

fn fixture_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    Ok(repo_root
        .join("target")
        .join("test-fixtures")
        .join("report"))
}

#[test]
#[allow(clippy::too_many_lines)]
fn golden_run_report_snapshot() -> Result<()> {
    let root = fixture_root()?;
    fs::create_dir_all(&root)?;

    let stage_report_path = root.join("stage_report.json");
    let retention_report_path = root.join("retention_report.json");
    let bank_report_path = root.join("bank_report.json");

    let stage_report = StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: "fastq.trim".to_string(),
        stage_version: 2,
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        metrics_path: root.join("metrics.json").display().to_string(),
        tool_invocation_path: root.join("tool_invocation.json").display().to_string(),
        effective_config_path: root.join("effective_config.json").display().to_string(),
        effective_config_hash: Some("sha256:config".to_string()),
        facts_row_id: Some("facts-1".to_string()),
        summary: serde_json::json!({"outputs": ["out.fastq.gz"]}),
        warnings: vec![],
        errors: vec![],
        outputs: vec!["out.fastq.gz".to_string()],
        subreports: vec![],
        log_paths: vec![],
    };
    fs::write(
        &stage_report_path,
        serde_json::to_vec_pretty(&stage_report)?,
    )?;

    let retention_report = RetentionReportV1 {
        schema_version: "bijux.retention_report.v1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        boundary: "pre/post".to_string(),
        numerator: serde_json::json!({"reads_out": 80, "bases_out": 800}),
        denominator: serde_json::json!({"reads_in": 100, "bases_in": 1000}),
        scope: "reads+bases".to_string(),
        condition: serde_json::json!({"min_len": 20}),
        parameters_json: serde_json::json!({"min_len": 20}),
        retention: Some(bijux_core::RetentionReportMetricV1 {
            value: 0.8,
            numerator_reads: 80,
            denominator_reads: 100,
            numerator_bases: 800,
            denominator_bases: 1000,
            definition: "reads_out / reads_in".to_string(),
            stage_boundary: "fastq.trim".to_string(),
            conditions: serde_json::json!({"min_len": 20}),
        }),
    };
    fs::write(
        &retention_report_path,
        serde_json::to_vec_pretty(&retention_report)?,
    )?;

    let bank_report = serde_json::json!({
        "schema_version": "bijux.bank_report.v1",
        "stage_id": "fastq.trim",
        "tool_id": "fastp",
        "banks": {
            "adapters": {
                "preset": "default",
                "hash": "sha256:adapters"
            }
        }
    });
    fs::write(&bank_report_path, serde_json::to_vec_pretty(&bank_report)?)?;

    let facts_path = root.join("facts.jsonl");
    let rows = vec![
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-1".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: Some("sha256:img".to_string()),
            params_hash: "params".to_string(),
            input_hash: "input".to_string(),
            output_hashes: vec!["out".to_string()],
            runtime_s: 1.0,
            memory_mb: 64.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({"adapters": "sha256:adapters"}),
            reads_in: Some(100),
            reads_out: Some(80),
            bases_in: Some(1000),
            bases_out: Some(800),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path.display().to_string(),
                "bank_report": bank_report_path.display().to_string()
            }),
            artifacts: serde_json::json!({}),
        },
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-1".to_string(),
            stage_id: "fastq.validate_pre".to_string(),
            tool_id: "fastqvalidator".to_string(),
            tool_version: "1.0".to_string(),
            image_digest: Some("sha256:img2".to_string()),
            params_hash: "params2".to_string(),
            input_hash: "input2".to_string(),
            output_hashes: vec!["out2".to_string()],
            runtime_s: 2.0,
            memory_mb: 32.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({}),
            reads_in: Some(50),
            reads_out: Some(50),
            bases_in: Some(500),
            bases_out: Some(500),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({
                "stage_report": stage_report_path.display().to_string()
            }),
            artifacts: serde_json::json!({}),
        },
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-1".to_string(),
            stage_id: "fastq.merge".to_string(),
            tool_id: "pear".to_string(),
            tool_version: "0.9".to_string(),
            image_digest: None,
            params_hash: "params3".to_string(),
            input_hash: "input3".to_string(),
            output_hashes: vec!["out3".to_string()],
            runtime_s: 3.0,
            memory_mb: 128.0,
            exit_code: 1,
            bank_hashes: serde_json::json!({}),
            reads_in: Some(100),
            reads_out: Some(90),
            bases_in: Some(1000),
            bases_out: Some(900),
            pairs_in: Some(50),
            pairs_out: Some(45),
            metrics: serde_json::json!({}),
            reports: serde_json::json!({
                "stage_report": stage_report_path.display().to_string()
            }),
            artifacts: serde_json::json!({}),
        },
    ];
    let mut facts_raw = String::new();
    for row in &rows {
        facts_raw.push_str(&serde_json::to_string(row)?);
        facts_raw.push('\n');
    }
    fs::write(&facts_path, facts_raw)?;

    let loaded = load_facts_jsonl(&facts_path)?;
    let report_path = write_run_report_from_facts(&root, &loaded)?;
    let report_raw = fs::read_to_string(report_path)?;
    let report_value: serde_json::Value = serde_json::from_str(&report_raw)?;
    let _: ReportSchemaV1 = serde_json::from_value(report_value.clone())?;

    let rendered = serde_json::to_string_pretty(&report_value)?;
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("run_report.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());

    let summary_path = root.join("run_summary.json");
    write_run_summary_from_facts(&summary_path, &loaded)?;
    let summary_raw = fs::read_to_string(&summary_path)?;
    let summary_value: serde_json::Value = serde_json::from_str(&summary_raw)?;
    let summary_snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("run_summary.json");
    let summary_snapshot = fs::read_to_string(&summary_snapshot_path)?;
    assert_eq!(
        serde_json::to_string_pretty(&summary_value)?.trim(),
        summary_snapshot.trim()
    );

    Ok(())
}
