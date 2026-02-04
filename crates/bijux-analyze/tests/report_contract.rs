use anyhow::Result;
use bijux_analyze::{load::load_facts, report::write_run_report_from_facts};
use bijux_core::{FactsRowV1, ReportSchemaV1, StageReportV1};
use std::fs;
use std::path::PathBuf;

#[test]
fn report_sections_exist_for_all_stages() -> Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let stage_dir = dir.path();
    let stages = bijux_domain_fastq::canonical_stage_order();

    let mut rows = Vec::new();
    for (idx, stage_id) in stages.iter().enumerate() {
        let stage_path = stage_dir.join(format!("stage_{idx}"));
        bijux_infra::ensure_dir(&stage_path)?;

        let metrics_path = stage_path.join("metrics.json");
        let invocation_path = stage_path.join("tool_invocation.json");
        let config_path = stage_path.join("effective_config.json");
        bijux_infra::write_bytes(&metrics_path, "{}")?;
        bijux_infra::write_bytes(&invocation_path, "{}")?;
        bijux_infra::write_bytes(&config_path, "{}")?;

        let stage_report = StageReportV1 {
            schema_version: "bijux.stage_report.v1".to_string(),
            stage_id: (*stage_id).to_string(),
            stage_version: 1,
            tool_id: "tool".to_string(),
            tool_version: "0.1".to_string(),
            metrics_path: metrics_path.display().to_string(),
            tool_invocation_path: invocation_path.display().to_string(),
            effective_config_path: config_path.display().to_string(),
            effective_config_hash: None,
            facts_row_id: None,
            summary: serde_json::json!({}),
            warnings: vec![],
            errors: vec![],
            invariants: vec![],
            verdict: Some(bijux_core::StageVerdictV1 {
                stage_id: (*stage_id).to_string(),
                verdict: bijux_core::InvariantStatusV1::Pass,
                reasons: Vec::new(),
                key_metrics: serde_json::json!({}),
            }),
            outputs: vec![],
            subreports: vec![],
            log_paths: vec![],
        };
        let stage_report_path = stage_path.join("stage_report.json");
        bijux_infra::write_bytes(
            &stage_report_path,
            serde_json::to_vec_pretty(&stage_report)?,
        )?;

        rows.push(FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-contract".to_string(),
            stage_id: (*stage_id).to_string(),
            tool_id: "tool".to_string(),
            tool_version: "0.1".to_string(),
            image_digest: Some("sha256:img".to_string()),
            trace_id: format!("trace-{idx}"),
            span_id: format!("span-{idx}"),
            params_hash: format!("params-{idx}"),
            input_hash: format!("input-{idx}"),
            output_hashes: vec![],
            runtime_s: 1.0,
            memory_mb: 1.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({}),
            reads_in: Some(10),
            reads_out: Some(10),
            bases_in: Some(100),
            bases_out: Some(100),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({"reads_in": 10}),
            reports: serde_json::json!({
                "stage_report": stage_report_path.display().to_string()
            }),
            artifacts: serde_json::json!({}),
        });
    }

    let facts_path = stage_dir.join("facts.jsonl");
    let mut facts_raw = String::new();
    for row in &rows {
        facts_raw.push_str(&serde_json::to_string(row)?);
        facts_raw.push('\n');
    }
    bijux_infra::write_bytes(&facts_path, facts_raw)?;
    let defaults = serde_json::json!({
        "pipeline_id": "fastq-to-fastq__default__v1",
        "tools": {},
        "params": {},
        "thresholds": {},
        "tool_provenance": {},
        "param_provenance": {},
        "assumptions": [],
        "citations": {},
    });
    bijux_infra::write_bytes(
        stage_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;
    let loaded = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let report_path = write_run_report_from_facts(stage_dir, &loaded)?;
    let report_raw = fs::read_to_string(report_path)?;
    let report: serde_json::Value = serde_json::from_str(&report_raw)?;
    let schema: ReportSchemaV1 = serde_json::from_value(report.clone())?;

    for section in schema.contract.required_sections {
        assert!(
            report.get(&section).is_some(),
            "missing required report section {section}"
        );
    }

    Ok(())
}

#[test]
fn report_schema_allows_unknown_fields() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("run_report.json");
    let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(snapshot_path)?)?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert("new_field".to_string(), serde_json::json!({"future": true}));
    }
    let _: ReportSchemaV1 = serde_json::from_value(value)?;
    Ok(())
}

#[test]
fn stage_sections_cover_all_executed_stages() -> Result<()> {
    let report = load_report_snapshot()?;
    let stages = report
        .get("stages")
        .and_then(|value| value.as_array())
        .ok_or_else(|| anyhow::anyhow!("missing stages"))?;
    let stage_ids: Vec<String> = stages
        .iter()
        .filter_map(|stage| stage.get("stage_id").and_then(|v| v.as_str()))
        .map(str::to_string)
        .collect();
    let stage_completeness = report
        .get("sections")
        .and_then(|value| value.get("stage_completeness"))
        .ok_or_else(|| anyhow::anyhow!("missing stage_completeness rows"))?;
    let rows = stage_completeness
        .get("rows")
        .and_then(|value| value.as_array())
        .or_else(|| stage_completeness.as_array())
        .ok_or_else(|| anyhow::anyhow!("missing stage_completeness rows"))?;
    let mut covered = std::collections::BTreeSet::new();
    for row in rows {
        if let Some(stage_id) = row.get("stage_id").and_then(|v| v.as_str()) {
            covered.insert(stage_id.to_string());
        }
    }
    for stage_id in stage_ids {
        assert!(
            covered.contains(&stage_id),
            "stage_completeness missing stage {stage_id}"
        );
    }
    Ok(())
}

fn load_report_snapshot() -> Result<serde_json::Value> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("run_report.json");
    let raw = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw)?)
}
