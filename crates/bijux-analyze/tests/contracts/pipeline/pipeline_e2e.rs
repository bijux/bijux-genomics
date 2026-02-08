use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_analyze::load::load_facts;
use bijux_analyze::report::write_run_report_from_facts;
use bijux_core::contract::canonical::canonicalize_truth_json;
use bijux_core::prelude::{InvariantStatusV1, StageId, StageVerdictV1};
use bijux_domain_bam::metrics::BamMetricsV1;
use bijux_domain_bam::BamStage;
use bijux_pipelines::registry::profile_by_id;
use bijux_pipelines::Domain;
use bijux_runtime::{FactsRowV1, StageReportV1};
use bijux_testkit::snapshot_name;
use serde_json::Value;

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
        bank_hashes: std::collections::BTreeMap::new(),
        reads_in: 100,
        reads_out: 80,
        bases_in: 1000,
        bases_out: 800,
        pairs_in: None,
        pairs_out: None,
        metrics: metrics_for_stage(stage_id),
        reports: bijux_runtime::FactsReportsV1 {
            stage_report: "stage_report.json".to_string(),
        },
        artifacts: bijux_runtime::FactsArtifactsV1 {
            metrics_envelope: "metrics_envelope.json".to_string(),
        },
    }
}

fn stage_report_for_stage(stage_id: &str) -> StageReportV1 {
    StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: "tool".to_string(),
        tool_version: "0.0.0".to_string(),
        started_at: "2024-01-01T00:00:00Z".to_string(),
        finished_at: "2024-01-01T00:00:01Z".to_string(),
        runtime_s: 1.0,
        memory_mb: 64.0,
        exit_code: 0,
        status: InvariantStatusV1::Pass,
        verdict: Some(StageVerdictV1::Pass),
        metrics_path: "metrics.json".to_string(),
        tool_invocation_path: "tool_invocation.json".to_string(),
        effective_config_path: "effective_config.json".to_string(),
    }
}

fn write_stage_artifacts(root: &Path, stage_id: &str) -> Result<()> {
    let stage_dir = root.join(stage_id);
    bijux_infra::ensure_dir(&stage_dir)?;
    fs::write(stage_dir.join("metrics.json"), serde_json::to_string_pretty(&metrics_for_stage(stage_id))?)?;
    fs::write(stage_dir.join("tool_invocation.json"), "{}")?;
    fs::write(stage_dir.join("effective_config.json"), "{}")?;
    let report = stage_report_for_stage(stage_id);
    fs::write(stage_dir.join("stage_report.json"), serde_json::to_string_pretty(&report)?)?;
    Ok(())
}

fn build_report(run_id: &str, profile_id: &str) -> Result<Value> {
    let profile = profile_by_id(profile_id).expect("profile exists");
    let temp = bijux_infra::temp_dir("pipeline-e2e")?;
    let root = temp.path();

    let facts_path = root.join("facts.jsonl");
    let mut facts = Vec::new();
    for stage in profile.stages.iter() {
        facts.push(fact_for_stage(stage, "tool", run_id));
        write_stage_artifacts(root, stage)?;
    }
    let facts_lines = facts
        .iter()
        .map(|row| serde_json::to_string(row).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&facts_path, facts_lines)?;

    let facts_loaded = load_facts(&facts_path)?;
    let report = write_run_report_from_facts(&facts_loaded, Domain::from(profile.domain))?;
    Ok(serde_json::to_value(report)?)
}

/// Snapshot locks fastq default pipeline report.
#[test]
fn pipeline_fastq_default_report_snapshot() -> Result<()> {
    let report = build_report("fastq-to-fastq__default__v1", "fastq-default")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__fastq-to-fastq__default__v1");
    insta::assert_json_snapshot!(name, json);
    Ok(())
}

/// Snapshot locks fastq-to-bam default pipeline report.
#[test]
fn pipeline_fastq_to_bam_default_report_snapshot() -> Result<()> {
    let report = build_report("fastq-to-bam__default__v1", "fastq-to-bam-default")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__fastq-to-bam__default__v1");
    insta::assert_json_snapshot!(name, json);
    Ok(())
}

/// Snapshot locks bam adna shotgun pipeline report.
#[test]
fn pipeline_bam_shotgun_report_snapshot() -> Result<()> {
    let report = build_report("bam-to-bam__adna_shotgun__v1", "bam-adna-shotgun")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__bam-to-bam__adna_shotgun__v1");
    insta::assert_json_snapshot!(name, json);
    Ok(())
}

/// Snapshot locks bam adna capture pipeline report.
#[test]
fn pipeline_bam_capture_report_snapshot() -> Result<()> {
    let report = build_report("bam-to-bam__adna_capture__v1", "bam-adna-capture")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__bam-to-bam__adna_capture__v1");
    insta::assert_json_snapshot!(name, json);
    Ok(())
}
