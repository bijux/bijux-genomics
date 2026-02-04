use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_analyze::{AnalyzeInput, AnalyzeMode, AnalyzeOptions, AnalyzeSources, RenderOptions};
use bijux_core::{FactsRowV1, InvariantStatusV1, StageReportV1, StageVerdictV1};
use bijux_domain_bam::metrics::BamMetricsV1;
use bijux_pipelines::registry::profile_by_id;
use bijux_pipelines::Domain;

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
        trace_id: format!("trace-{run_id}"),
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
        artifacts: serde_json::json!({
            "metrics_envelope": "metrics_envelope.json"
        }),
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

fn write_facts(base_dir: &Path, profile: &bijux_pipelines::PipelineProfile) -> Result<PathBuf> {
    let run_id = profile.id.as_str();
    let mut rows = Vec::new();
    for (idx, node) in profile.graph.iter().enumerate() {
        let tool = profile
            .defaults
            .tools
            .get(&node.stage_id)
            .map_or("unknown", String::as_str);
        let stage_dir = base_dir.join(format!("stage_{idx}"));
        bijux_infra::ensure_dir(&stage_dir)?;
        let stage_report_path = write_stage_report(&stage_dir, &node.stage_id, tool)?;
        let mut row = fact_for_stage(&node.stage_id, tool, run_id);
        row.reports = serde_json::json!({
            "stage_report": stage_report_path.display().to_string()
        });
        rows.push(row);
    }
    let facts_path = base_dir.join("facts.jsonl");
    let mut raw = String::new();
    for row in &rows {
        raw.push_str(&serde_json::to_string(row)?);
        raw.push('\n');
    }
    bijux_infra::write_bytes(&facts_path, raw)?;
    Ok(facts_path)
}

fn write_defaults(base_dir: &Path, profile: &bijux_pipelines::PipelineProfile) -> Result<()> {
    let defaults = profile.defaults_ledger();
    bijux_infra::write_bytes(
        base_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;
    Ok(())
}

fn write_manifest(
    base_dir: &Path,
    profile: &bijux_pipelines::PipelineProfile,
    domain: Domain,
) -> Result<()> {
    let run_id = profile.id.as_str();
    let manifest = match domain {
        Domain::Cross => serde_json::json!({
            "schema_version": "bijux.run_manifest.v2",
            "run_id": run_id,
            "profile_id": profile.id.as_str(),
            "domain_transitions": [{
                "from": "fastq",
                "to": "bam",
                "boundary": "run_artifacts/boundaries/alignment_boundary.json"
            }],
            "boundaries": [],
        }),
        _ => serde_json::json!({
            "schema_version": "bijux.run_manifest.v1",
            "run_id": run_id,
            "pipeline_id": profile.id.as_str(),
            "stages": profile.graph.iter().map(|node| node.stage_id.clone()).collect::<Vec<_>>(),
        }),
    };
    bijux_infra::atomic_write_json(&base_dir.join("run_manifest.json"), &manifest)?;
    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    Ok(bijux_infra::hash_file_sha256(path)?)
}

fn run_pipeline_case(domain: Domain, pipeline_id: &str) -> Result<(String, String)> {
    let profile = profile_by_id(domain, pipeline_id)?;
    let tmp = tempfile::tempdir()?;
    let run_id = pipeline_id;
    let layout = bijux_infra::run_layout_paths(tmp.path(), run_id);
    bijux_infra::ensure_dir(&layout.artifacts_dir)?;
    bijux_infra::ensure_dir(&layout.logs_dir)?;
    bijux_infra::ensure_dir(&layout.tmp_dir)?;
    assert!(layout
        .run_dir
        .ends_with(PathBuf::from(bijux_infra::RUN_LAYOUT_CONTRACT.runs_dir).join(run_id)));

    write_defaults(&layout.artifacts_dir, &profile)?;
    write_manifest(&layout.artifacts_dir, &profile, domain)?;
    let facts_path = write_facts(&layout.artifacts_dir, &profile)?;

    let input = AnalyzeInput {
        run_id: Some(run_id.to_string()),
        sources: AnalyzeSources::FactsJsonl(facts_path),
        options: AnalyzeOptions {
            mode: AnalyzeMode::Report,
            strict: true,
            render: RenderOptions {
                json: true,
                html: true,
                output_dir: None,
            },
        },
    };
    let output = bijux_analyze::analyze_run(&input)?;
    let report_path = output
        .report_json
        .ok_or_else(|| anyhow::anyhow!("missing report.json"))?;
    let bundle_index = layout
        .artifacts_dir
        .join("report_bundle")
        .join("index.html");
    assert!(
        bundle_index.exists(),
        "missing report bundle index for {pipeline_id}"
    );

    Ok((hash_file(&report_path)?, hash_file(&bundle_index)?))
}

#[test]
fn golden_spine_blessed_pipelines() -> Result<()> {
    let cases = [
        (Domain::Fastq, "fastq-to-fastq__default__v1"),
        (Domain::Cross, "fastq-to-bam__default__v1"),
        (Domain::Bam, "bam-to-bam__adna_shotgun__v1"),
    ];

    for (domain, pipeline_id) in cases {
        let (report_hash_a, bundle_hash_a) = run_pipeline_case(domain, pipeline_id)?;
        let (report_hash_b, bundle_hash_b) = run_pipeline_case(domain, pipeline_id)?;
        assert_eq!(
            report_hash_a, report_hash_b,
            "report hash mismatch for {pipeline_id}"
        );
        assert_eq!(
            bundle_hash_a, bundle_hash_b,
            "bundle hash mismatch for {pipeline_id}"
        );
    }

    Ok(())
}
