use std::fs;
use std::path::Path;

use anyhow::Result;
use bijux_dna_analyze::report::write_run_report_from_facts;
use bijux_dna_core::contract::canonical::canonicalize_truth_json;
use bijux_dna_core::prelude::{InvariantStatusV1, StageVerdictV1};
use bijux_dna_domain_bam::metrics::BamMetricsV1;
use bijux_dna_pipelines::registry::profile_by_id;
use bijux_dna_pipelines::Domain;
use bijux_dna_runtime::{FactsRowV1, StageReportV1};
use insta::Settings;
use serde_json::Value;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-analyze__{group}__{name}")
}

fn is_optional_bam_downstream(stage_id: &str) -> bool {
    matches!(stage_id, "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship")
}

fn feature_stable_profile(
    mut profile: bijux_dna_pipelines::PipelineProfile,
) -> bijux_dna_pipelines::PipelineProfile {
    profile.capabilities.required_stages.retain(|stage_id| !is_optional_bam_downstream(stage_id));
    profile.defaults.tools.retain(|stage_id, _| !is_optional_bam_downstream(stage_id.as_str()));
    profile.defaults.params.retain(|stage_id, _| !is_optional_bam_downstream(stage_id.as_str()));
    profile
        .defaults
        .rationales
        .retain(|stage_id, _| !is_optional_bam_downstream(stage_id.as_str()));
    profile
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
        tool_version: "99.99.99+fixture".to_string(),
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
            "stage_report": format!("{stage_id}/stage_report.json")
        }),
        artifacts: serde_json::json!({
            "metrics_envelope": format!("{stage_id}/metrics_envelope.json")
        }),
    }
}

fn stage_report_for_stage(stage_id: &str) -> StageReportV1 {
    StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version: 1,
        tool_id: "tool".to_string(),
        tool_version: "99.99.99+fixture".to_string(),
        metrics_path: "metrics.json".to_string(),
        tool_invocation_path: "tool_invocation.json".to_string(),
        effective_config_path: "effective_config.json".to_string(),
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
    }
}

fn write_stage_artifacts(root: &Path, stage_id: &str) -> Result<()> {
    let stage_dir = root.join(stage_id);
    bijux_dna_infra::ensure_dir(&stage_dir)?;
    fs::write(
        stage_dir.join("metrics.json"),
        serde_json::to_string_pretty(&metrics_for_stage(stage_id))?,
    )?;
    fs::write(stage_dir.join("tool_invocation.json"), "{}")?;
    fs::write(stage_dir.join("effective_config.json"), "{}")?;
    fs::write(stage_dir.join("metrics_envelope.json"), "{}")?;
    let report = stage_report_for_stage(stage_id);
    fs::write(stage_dir.join("stage_report.json"), serde_json::to_string_pretty(&report)?)?;
    Ok(())
}

fn build_report(domain: Domain, pipeline_id: &str) -> Result<Value> {
    let profile = feature_stable_profile(
        profile_by_id(domain, pipeline_id).unwrap_or_else(|err| panic!("profile exists: {err}")),
    );
    let paths = bijux_dna_testkit::TestPaths::new(&format!("pipeline-e2e-{pipeline_id}"));
    let root = paths.root();

    let mut stages = profile.capabilities.required_stages.clone();
    stages.sort_unstable();
    let mut facts = Vec::new();
    for stage in &stages {
        let stage_key = bijux_dna_core::ids::StageId::new(stage);
        let tool_id = profile.defaults.tools.get(&stage_key).map_or("tool", |tool| tool.as_str());
        facts.push(fact_for_stage(stage, tool_id, pipeline_id));
        write_stage_artifacts(root, stage)?;
    }
    bijux_dna_infra::atomic_write_json(
        &root.join("defaults_ledger.json"),
        &profile.defaults_ledger(),
    )?;

    let report_path = write_run_report_from_facts(root, &facts)?;
    let report = fs::read_to_string(report_path)?;
    Ok(serde_json::from_str(&report)?)
}

fn snapshot_settings() -> Settings {
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings
}

/// Snapshot locks fastq default pipeline report.
#[test]
fn pipeline_fastq_default_report_snapshot() -> Result<()> {
    let report = build_report(Domain::Fastq, "fastq-to-fastq__default__v1")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__fastq-to-fastq__default__v1");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

/// Snapshot locks fastq-to-bam default pipeline report.
///
/// Nondeterminism vectors eliminated:
/// - Explicit stage sorting before report assembly to prevent snapshot drift from input order.
/// - Per-test unique temp roots via `TestPaths` under `TEST_TMP_DIR`.
#[test]
fn pipeline_fastq_to_bam_default_report_snapshot() -> Result<()> {
    let report = build_report(Domain::Cross, "fastq-to-bam__default__v1")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__fastq-to-bam__default__v1");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

/// Snapshot locks bam adna shotgun pipeline report.
///
/// Nondeterminism vectors eliminated:
/// - Explicit stage sorting before report assembly to prevent snapshot drift from input order.
/// - Per-test unique temp roots via `TestPaths` under `TEST_TMP_DIR`.
#[test]
fn pipeline_bam_shotgun_report_snapshot() -> Result<()> {
    let report = build_report(Domain::Bam, "bam-to-bam__adna_shotgun__v1")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__bam-to-bam__adna_shotgun__v1");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

/// Snapshot locks bam adna capture pipeline report.
///
/// Nondeterminism vectors eliminated:
/// - Explicit stage sorting before report assembly to prevent snapshot drift from input order.
/// - Per-test unique temp roots via `TestPaths` under `TEST_TMP_DIR`.
#[test]
fn pipeline_bam_capture_report_snapshot() -> Result<()> {
    let report = build_report(Domain::Bam, "bam-to-bam__adna_capture__v1")?;
    let json = canonicalize_truth_json(&report);
    let name = snapshot_name("contracts", "pipeline__bam-to-bam__adna_capture__v1");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

/// Determinism contract for historical snapshot flakes in `pipeline_e2e` report generation.
#[test]
fn pipeline_reports_are_stable_across_repeated_builds() -> Result<()> {
    let cases = [
        (Domain::Cross, "fastq-to-bam__default__v1"),
        (Domain::Bam, "bam-to-bam__adna_shotgun__v1"),
        (Domain::Bam, "bam-to-bam__adna_capture__v1"),
    ];
    for (domain, pipeline_id) in cases {
        let a = bijux_dna_testkit::snapshot_normalize_json(&canonicalize_truth_json(
            &build_report(domain, pipeline_id)?,
        ));
        let b = bijux_dna_testkit::snapshot_normalize_json(&canonicalize_truth_json(
            &build_report(domain, pipeline_id)?,
        ));
        assert_eq!(a, b, "pipeline report must be deterministic for {pipeline_id}");
    }
    Ok(())
}

#[test]
fn feature_stable_profile_prunes_optional_bam_downstream_stages() -> Result<()> {
    let profile =
        feature_stable_profile(profile_by_id(Domain::Bam, "bam-to-bam__adna_shotgun__v1")?);
    let stage_ids = &profile.capabilities.required_stages;

    assert!(
        !stage_ids.iter().any(|stage_id| is_optional_bam_downstream(stage_id)),
        "pipeline_e2e snapshots must stay stable when workspace feature unification enables optional BAM downstream stages"
    );
    assert!(
        profile
            .defaults
            .tools
            .keys()
            .all(|stage_id| !is_optional_bam_downstream(stage_id.as_str())),
        "tool defaults must stay aligned with the pruned BAM snapshot stage set"
    );

    Ok(())
}
