use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_analyze::load::load_facts;
use bijux_analyze::report::write_run_report_from_facts;
use bijux_core::prelude::{InvariantStatusV1, StageVerdictV1};
use bijux_domain_bam::metrics::BamMetricsV1;
use bijux_pipelines::registry::profile_by_id;
use bijux_pipelines::Domain;
use bijux_runtime::{FactsRowV1, StageReportV1};

fn metrics_for_stage(stage_id: &str) -> serde_json::Value {
    if stage_id.starts_with("bam.") {
        serde_json::to_value(BamMetricsV1::empty()).unwrap_or(serde_json::json!({}))
    } else if stage_id.starts_with("fastq.") {
        serde_json::json!({"reads_in": 100, "reads_out": 80, "bases_in": 1000, "bases_out": 800})
    } else {
        serde_json::json!({})
    }
}

fn fact_for_stage(stage_id: &str, tool_id: &str, run_id: &str) -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: run_id.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: "0.0.0".to_string(),
        image_digest: Some("sha256:img".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: format!("span-{stage_id}"),
        params_hash: "params".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        runtime_s: 1.0,
        memory_mb: 64.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(100),
        reads_out: Some(100),
        bases_in: Some(1000),
        bases_out: Some(1000),
        pairs_in: None,
        pairs_out: None,
        metrics: metrics_for_stage(stage_id),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }
}

fn write_stage_report(stage_dir: &Path, stage_id: &str, tool_id: &str) -> Result<PathBuf> {
    let metrics_path = stage_dir.join("metrics.json");
    let invocation_path = stage_dir.join("tool_invocation.json");
    let config_path = stage_dir.join("effective_config.json");
    bijux_infra::write_bytes(&metrics_path, "{}")?;
    bijux_infra::write_bytes(&invocation_path, "{}")?;
    bijux_infra::write_bytes(&config_path, "{}")?;
    let stage_report = StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version: 1,
        tool_id: tool_id.to_string(),
        tool_version: "0.0.0".to_string(),
        metrics_path: metrics_path.display().to_string(),
        tool_invocation_path: invocation_path.display().to_string(),
        effective_config_path: config_path.display().to_string(),
        effective_config_hash: None,
        facts_row_id: None,
        summary: serde_json::json!({}),
        warnings: Vec::new(),
        errors: Vec::new(),
        invariants: Vec::new(),
        verdict: Some(StageVerdictV1 {
            stage_id: stage_id.to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: Vec::new(),
            key_metrics: serde_json::json!({}),
        }),
        outputs: Vec::new(),
        subreports: Vec::new(),
        log_paths: Vec::new(),
    };
    let stage_report_path = stage_dir.join("stage_report.json");
    bijux_infra::write_bytes(
        &stage_report_path,
        serde_json::to_vec_pretty(&stage_report)?,
    )?;
    Ok(stage_report_path)
}

fn write_pipeline_report(domain: Domain, pipeline_id: &str) -> Result<serde_json::Value> {
    let profile = profile_by_id(domain, pipeline_id)?;
    let run_id = pipeline_id;
    let mut rows = Vec::new();
    let dir = tempfile::tempdir()?;
    let base_dir = dir.path().join("pipeline");
    bijux_infra::ensure_dir(&base_dir)?;
    let id_catalog = match profile.id.as_str() {
        "fastq-to-fastq__default__v1" | "fastq-to-fastq__minimal__v1" => {
            bijux_planner_fastq::fastq_pipeline_id_catalog(profile.id.as_str())
        }
        "fastq-to-bam__default__v1" | "fastq-to-bam__adna_shotgun__v1" => {
            bijux_planner_fastq::cross_fastq_to_bam_id_catalog(profile.id.as_str())
        }
        "bam-to-bam__default__v1"
        | "bam-to-bam__adna_shotgun__v1"
        | "bam-to-bam__adna_capture__v1" => {
            bijux_planner_bam::pipeline_id_catalog(profile.id.as_str())
        }
        _ => Vec::new(),
    };
    for (idx, stage_id) in id_catalog.iter().enumerate() {
        let stage_key = bijux_core::ids::StageId::new(stage_id.clone());
        let tool = profile
            .defaults
            .tools
            .get(&stage_key)
            .map_or("unknown", |tool| tool.as_str());
        let stage_dir = base_dir.join(format!("stage_{idx}"));
        bijux_infra::ensure_dir(&stage_dir)?;
        let stage_report_path = write_stage_report(&stage_dir, stage_id, tool)?;
        let mut row = fact_for_stage(stage_id, tool, run_id);
        row.reports = serde_json::json!({
            "stage_report": stage_report_path.display().to_string()
        });
        rows.push(row);
    }
    let facts_path = base_dir.join("facts.jsonl");
    let mut facts_raw = String::new();
    for row in &rows {
        facts_raw.push_str(&serde_json::to_string(row)?);
        facts_raw.push('\n');
    }
    bijux_infra::write_bytes(&facts_path, facts_raw)?;
    let defaults = profile.defaults_ledger();
    bijux_infra::write_bytes(
        base_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;
    let loaded = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let report_path = write_run_report_from_facts(&base_dir, &loaded)?;
    let report_raw = std::fs::read_to_string(report_path)?;
    Ok(serde_json::from_str(&report_raw)?)
}

fn assert_no_absolute_paths(value: &serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            assert!(
                !s.starts_with('/') || s.starts_with("//"),
                "absolute path found: {s}"
            );
            assert!(!s.contains(":\\"), "windows absolute path found: {s}");
        }
        serde_json::Value::Array(items) => {
            for item in items {
                assert_no_absolute_paths(item);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, value) in map {
                assert_no_absolute_paths(value);
            }
        }
        _ => {}
    }
}

#[test]
fn report_has_no_absolute_paths() -> Result<()> {
    let report = write_pipeline_report(Domain::Fastq, "fastq-to-fastq__default__v1")?;
    assert_no_absolute_paths(&report);
    Ok(())
}
