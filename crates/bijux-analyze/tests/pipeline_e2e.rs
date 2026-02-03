use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_analyze::load::load_facts;
use bijux_analyze::report::write_run_report_from_facts;
use bijux_core::FactsRowV1;
use bijux_domain_bam::metrics::BamMetricsV1;
use bijux_pipelines::registry::profile_by_id;
use bijux_pipelines::Domain;

fn fixture_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    Ok(repo_root
        .join("target")
        .join("test-fixtures")
        .join("pipelines"))
}

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
        reads_out: Some(80),
        bases_in: Some(1000),
        bases_out: Some(800),
        pairs_in: None,
        pairs_out: None,
        metrics: metrics_for_stage(stage_id),
        reports: serde_json::json!({
            "stage_report": "/tmp/stage_report.json",
            "retention_report": "/tmp/retention_report.json",
            "bank_report": "/tmp/bank_report.json"
        }),
        artifacts: serde_json::json!({
            "metrics_envelope": "/tmp/metrics_envelope.json"
        }),
    }
}

fn write_pipeline_report(domain: Domain, pipeline_id: &str) -> Result<serde_json::Value> {
    let profile = profile_by_id(domain, pipeline_id)?;
    let run_id = pipeline_id;
    let mut rows = Vec::new();
    for node in &profile.graph {
        let tool = profile
            .defaults
            .tools
            .get(&node.stage_id)
            .map(|tool| tool.as_str())
            .unwrap_or("unknown");
        rows.push(fact_for_stage(&node.stage_id, tool, run_id));
    }
    let root = fixture_root()?;
    let dir = root.join(pipeline_id);
    fs::create_dir_all(&dir)?;
    let facts_path = dir.join("facts.jsonl");
    let mut facts_raw = String::new();
    for row in &rows {
        facts_raw.push_str(&serde_json::to_string(row)?);
        facts_raw.push('\n');
    }
    fs::write(&facts_path, facts_raw)?;
    let loaded = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let report_path = write_run_report_from_facts(&dir, &loaded)?;
    let report_raw = fs::read_to_string(report_path)?;
    Ok(serde_json::from_str(&report_raw)?)
}

#[test]
fn pipeline_fastq_default_report_snapshot() -> Result<()> {
    let report = write_pipeline_report(Domain::Fastq, "fastq-to-fastq__default__v1")?;
    insta::assert_json_snapshot!("pipeline__fastq-to-fastq__default__v1", report);
    Ok(())
}

#[test]
fn pipeline_fastq_to_bam_default_report_snapshot() -> Result<()> {
    let report = write_pipeline_report(Domain::Cross, "fastq-to-bam__default__v1")?;
    insta::assert_json_snapshot!("pipeline__fastq-to-bam__default__v1", report);
    Ok(())
}

#[test]
fn pipeline_bam_adna_shotgun_report_snapshot() -> Result<()> {
    let report = write_pipeline_report(Domain::Bam, "bam-to-bam__adna_shotgun__v1")?;
    insta::assert_json_snapshot!("pipeline__bam-to-bam__adna_shotgun__v1", report);
    Ok(())
}

#[test]
fn pipeline_bam_adna_capture_report_snapshot() -> Result<()> {
    let report = write_pipeline_report(Domain::Bam, "bam-to-bam__adna_capture__v1")?;
    insta::assert_json_snapshot!("pipeline__bam-to-bam__adna_capture__v1", report);
    Ok(())
}
