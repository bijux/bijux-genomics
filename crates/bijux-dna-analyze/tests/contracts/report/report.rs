use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_analyze::{load::load_facts, report::write_run_report_from_facts};
use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::{InvariantStatusV1, StageVerdictV1, ToolConstraints};
use bijux_dna_runtime::{
    EffectiveConfigV1, FactsRowV1, ReportSchemaV1, RetentionReportV1, StageReportV1,
};

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-analyze__{group}__{name}")
}

fn fixture_root() -> Result<tempfile::TempDir> {
    bijux_dna_infra::temp_dir("analyze-report-fixtures")
        .map_err(|err| anyhow::anyhow!(err.to_string()))
}

fn fixture_case_dir(root: &std::path::Path, suffix: &str) -> PathBuf {
    root.join(suffix)
}

fn write_report_fixture(
    root: &std::path::Path,
    suffix: &str,
    rows: &[FactsRowV1],
) -> Result<serde_json::Value> {
    let dir = root.join(suffix);
    bijux_dna_infra::ensure_dir(&dir)?;
    let facts_path = dir.join("facts.jsonl");
    let mut facts_raw = String::new();
    for row in rows {
        facts_raw.push_str(&serde_json::to_string(row)?);
        facts_raw.push('\n');
    }
    bijux_dna_infra::write_bytes(&facts_path, facts_raw)?;
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
    bijux_dna_infra::write_bytes(
        dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;
    let loaded = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let report_path = write_run_report_from_facts(&dir, &loaded)?;
    let report_raw = fs::read_to_string(report_path)?;
    Ok(serde_json::from_str(&report_raw)?)
}

#[allow(clippy::too_many_lines)]
fn base_reports(root: &std::path::Path) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let stage_report_path = root.join("stage_report.json");
    let retention_report_path = root.join("retention_report.json");
    let bank_report_path = root.join("bank_report.json");

    let stage_report = StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        stage_version: 2,
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        metrics_path: root.join("metrics.json").display().to_string(),
        tool_invocation_path: root.join("tool_invocation.json").display().to_string(),
        effective_config_path: root.join("effective_config.json").display().to_string(),
        effective_config_hash: Some("sha256:config".to_string()),
        facts_row_id: Some("facts-1".to_string()),
        summary: serde_json::json!({"outputs": ["out.fastq.gz"]}),
        warnings: vec!["low_q".to_string()],
        errors: vec![],
        invariants: vec![],
        verdict: Some(StageVerdictV1 {
            stage_id: "fastq.trim_reads".to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: Vec::new(),
            key_metrics: serde_json::json!({}),
        }),
        outputs: vec!["out.fastq.gz".to_string()],
        subreports: vec![],
        log_paths: vec![],
    };
    bijux_dna_infra::write_bytes(&stage_report_path, serde_json::to_vec_pretty(&stage_report)?)?;

    let effective_config = EffectiveConfigV1 {
        schema_version: "bijux.effective_config.v1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        stage_version: 2,
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:img".to_string()),
        runner: "docker".to_string(),
        platform: "linux".to_string(),
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        parameters_json: serde_json::json!({"min_len": 20}),
        parameters_json_normalized: serde_json::json!({"min_len": 20}),
        effective_params_json: serde_json::json!({
            "paired_mode": "single_end",
            "threads": 1,
            "min_len": 20,
            "adapter_policy": "bank"
        }),
        effective_params_json_normalized: serde_json::json!({
            "adapter_policy": "bank",
            "min_len": 20,
            "paired_mode": "single_end",
            "threads": 1
        }),
        adapter_bank: None,
        banks: None,
        bank_assets: None,
    };
    bijux_dna_infra::write_bytes(
        root.join("effective_config.json"),
        serde_json::to_vec_pretty(&effective_config)?,
    )?;

    let tool_invocation = ToolInvocationV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: bijux_dna_core::contract::ContractVersion::v1(),
        stage_id: bijux_dna_core::ids::StageId::from_static("fastq.trim_reads"),
        tool_id: bijux_dna_core::ids::ToolId::from_static("fastp"),
        tool_version: "0.23.4".to_string(),
        resolved_tool_version: Some("0.23.4".to_string()),
        image_digest: "sha256:img".to_string(),
        runner_kind: "docker".to_string(),
        platform: "linux".to_string(),
        parameters_json: serde_json::json!({"min_len": 20}),
        parameters_json_normalized: serde_json::json!({"min_len": 20}),
        effective_params_json: serde_json::json!({
            "paired_mode": "single_end",
            "threads": 1,
            "min_len": 20,
            "adapter_policy": "bank"
        }),
        effective_params_json_normalized: serde_json::json!({
            "adapter_policy": "bank",
            "min_len": 20,
            "paired_mode": "single_end",
            "threads": 1
        }),
        params_provenance: serde_json::json!({}),
        params_provenance_normalized: serde_json::json!({}),
        adapter_bank: None,
        banks: None,
        bank_assets: None,
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        environment: std::collections::BTreeMap::new(),
        input_hashes: vec!["input".to_string()],
        output_hashes: vec!["out".to_string()],
        executed_command: Some("fastp --in1 reads.fastq.gz".to_string()),
    };
    bijux_dna_infra::write_bytes(
        root.join("tool_invocation.json"),
        serde_json::to_vec_pretty(&tool_invocation)?,
    )?;

    let retention_report = RetentionReportV1 {
        schema_version: "bijux.retention_report.v1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        boundary: "pre/post".to_string(),
        numerator: serde_json::json!({"reads_out": 80, "bases_out": 800}),
        denominator: serde_json::json!({"reads_in": 100, "bases_in": 1000}),
        units: "reads".to_string(),
        scope: "reads+bases".to_string(),
        condition: serde_json::json!({"min_len": 20}),
        parameters_json: serde_json::json!({"min_len": 20}),
    };
    bijux_dna_infra::write_bytes(
        &retention_report_path,
        serde_json::to_vec_pretty(&retention_report)?,
    )?;

    let bank_report = serde_json::json!({
        "schema_version": "bijux.bank_report.v1",
        "stage_id": "fastq.trim_reads",
        "tool_id": "fastp",
        "banks": {
            "adapters": {
                "preset": "default",
                "hash": "sha256:adapters",
                "entries": [
                    { "id": "A1", "sequence": "ACGT", "enabled": true }
                ]
            },
            "polyx": {
                "preset": "illumina_twocolor",
                "hash": "sha256:polyx",
                "entries": [
                    { "id": "polyG", "enabled": true }
                ]
            },
            "contaminants": {
                "preset": "illumina_default",
                "hash": "sha256:contam",
                "entries": [
                    { "id": "phix", "enabled": true }
                ]
            }
        }
    });
    bijux_dna_infra::write_bytes(&bank_report_path, serde_json::to_vec_pretty(&bank_report)?)?;

    Ok((stage_report_path, retention_report_path, bank_report_path))
}

#[test]
#[allow(clippy::too_many_lines)]
fn golden_run_report_snapshot_happy_path() -> Result<()> {
    let root = fixture_root()?;
    bijux_dna_infra::ensure_dir(root.path())?;
    let case_dir = fixture_case_dir(root.path(), "happy");
    let (stage_report_path, retention_report_path, bank_report_path) = base_reports(&case_dir)?;

    let rows = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:img".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "params".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        runtime_s: 1.0,
        memory_mb: 64.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({
            "adapters": "sha256:adapters",
            "polyx": "sha256:polyx",
            "contaminants": "sha256:contam"
        }),
        reads_in: Some(100),
        reads_out: Some(80),
        bases_in: Some(1000),
        bases_out: Some(800),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({"reads_in": 100}),
        reports: serde_json::json!({
            "stage_report": stage_report_path.display().to_string(),
            "retention_report": retention_report_path.display().to_string(),
            "bank_report": bank_report_path.display().to_string()
        }),
        artifacts: serde_json::json!({}),
    }];

    let report_value = write_report_fixture(root.path(), "happy", &rows)?;
    let _: ReportSchemaV1 = serde_json::from_value(report_value.clone())?;
    let snapshot_file = format!("{}.json", snapshot_name("schemas", "run_report"));
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(snapshot_file);
    if std::env::var("UPDATE_GOLDEN_REPORT").is_ok() {
        let pretty = serde_json::to_string_pretty(&report_value)?;
        fs::write(&snapshot_path, pretty)?;
        return Ok(());
    }
    let snapshot_raw = fs::read_to_string(&snapshot_path)?;
    let snapshot_value: serde_json::Value = serde_json::from_str(&snapshot_raw)?;
    assert_eq!(report_value, snapshot_value);

    Ok(())
}

#[test]
fn report_includes_sections_block() -> Result<()> {
    let root = fixture_root()?;
    let case_dir = fixture_case_dir(root.path(), "sections");
    let (stage_report_path, retention_report_path, bank_report_path) = base_reports(&case_dir)?;
    let rows = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-sections".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:img".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "params".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        runtime_s: 1.0,
        memory_mb: 64.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({
            "adapters": "sha256:adapters",
            "polyx": "sha256:polyx",
            "contaminants": "sha256:contam"
        }),
        reads_in: Some(100),
        reads_out: Some(80),
        bases_in: Some(1000),
        bases_out: Some(800),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({"reads_in": 100}),
        reports: serde_json::json!({
            "stage_report": stage_report_path.display().to_string(),
            "retention_report": retention_report_path.display().to_string(),
            "bank_report": bank_report_path.display().to_string()
        }),
        artifacts: serde_json::json!({}),
    }];
    let report_value = write_report_fixture(root.path(), "sections", &rows)?;
    let Some(sections) = report_value.get("sections").and_then(|value| value.as_object()) else {
        panic!("sections block missing")
    };
    for key in ["qc", "trimming", "filtering", "contamination", "retention", "failures"] {
        assert!(sections.contains_key(key), "missing section {key}");
    }
    Ok(())
}

#[test]
fn golden_run_report_snapshot_tool_failure() -> Result<()> {
    let root = fixture_root()?;
    let case_dir = fixture_case_dir(root.path(), "failure");
    let (stage_report_path, _, _) = base_reports(&case_dir)?;
    let rows = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-2".to_string(),
        stage_id: "fastq.merge_pairs".to_string(),
        tool_id: "pear".to_string(),
        tool_version: "0.9".to_string(),
        image_digest: Some("sha256:img2".to_string()),
        trace_id: "trace-2".to_string(),
        span_id: "span-2".to_string(),
        params_hash: "params2".to_string(),
        input_hash: "input2".to_string(),
        output_hashes: vec!["out2".to_string()],
        runtime_s: 2.0,
        memory_mb: 128.0,
        exit_code: 1,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(100),
        reads_out: Some(90),
        bases_in: Some(1000),
        bases_out: Some(900),
        pairs_in: Some(50),
        pairs_out: Some(45),
        metrics: serde_json::json!({"reads_in": 100}),
        reports: serde_json::json!({
            "stage_report": stage_report_path.display().to_string()
        }),
        artifacts: serde_json::json!({}),
    }];
    let report_value = write_report_fixture(root.path(), "failure", &rows)?;
    assert_eq!(report_value["completeness"]["status"], "incomplete");
    Ok(())
}

#[test]
fn golden_run_report_snapshot_missing_metrics() -> Result<()> {
    let root = fixture_root()?;
    let case_dir = fixture_case_dir(root.path(), "missing");
    let (stage_report_path, _, _) = base_reports(&case_dir)?;
    let rows = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-3".to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: "fastqvalidator".to_string(),
        tool_version: "1.0".to_string(),
        image_digest: Some("sha256:img3".to_string()),
        trace_id: "trace-3".to_string(),
        span_id: "span-3".to_string(),
        params_hash: "params3".to_string(),
        input_hash: "input3".to_string(),
        output_hashes: vec!["out3".to_string()],
        runtime_s: 3.0,
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
    }];
    let report_value = write_report_fixture(root.path(), "missing", &rows)?;
    assert_eq!(report_value["completeness"]["status"], "incomplete");
    Ok(())
}

#[test]
fn report_provenance_is_complete() -> Result<()> {
    let root = fixture_root()?;
    let case_dir = fixture_case_dir(root.path(), "provenance");
    let (stage_report_path, retention_report_path, bank_report_path) = base_reports(&case_dir)?;
    let rows = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-4".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:img".to_string()),
        trace_id: "trace-4".to_string(),
        span_id: "span-4".to_string(),
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
        metrics: serde_json::json!({"reads_in": 100}),
        reports: serde_json::json!({
            "stage_report": stage_report_path.display().to_string(),
            "retention_report": retention_report_path.display().to_string(),
            "bank_report": bank_report_path.display().to_string()
        }),
        artifacts: serde_json::json!({}),
    }];

    let report_value = write_report_fixture(root.path(), "provenance", &rows)?;
    for entry in report_value["provenance"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing provenance"))?
    {
        assert!(!entry["tool_id"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["tool_version"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["params_hash"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["image_digest"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["trace_id"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["span_id"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["bank_hashes"].is_null());
    }
    Ok(())
}

fn observation_fact(stage_report: Option<&std::path::Path>, run_id: &str) -> Result<FactsRowV1> {
    Ok(serde_json::from_value(serde_json::json!({
        "schema_version": "bijux.facts.v1", "run_id": run_id,
        "stage_id": "fastq.trim_reads", "tool_id": "fastp", "tool_version": "0.23.4",
        "image_digest": "sha256:img", "trace_id": "trace", "span_id": "span",
        "params_hash": "parameters", "input_hash": "input", "output_hashes": ["output"],
        "runtime_s": 1.0, "memory_mb": 10.0, "exit_code": 0, "bank_hashes": {},
        "reads_in": 100, "reads_out": 100, "bases_in": 1000, "bases_out": 1000,
        "pairs_in": null, "pairs_out": null,
        "metrics": {"reads_in": 100, "reads_out": 100, "bases_in": 1000, "bases_out": 1000},
        "reports": stage_report.map_or_else(|| serde_json::json!({}), |path|
            serde_json::json!({"stage_report": path})),
        "artifacts": {"metrics_envelope": "metrics_envelope.json"}
    }))?)
}

fn observation_defaults(root: &std::path::Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(root)?;
    bijux_dna_infra::write_bytes(
        root.join("defaults_ledger.json"),
        serde_json::to_vec(&serde_json::json!({
            "pipeline_id": "fastq-to-fastq__default__v1", "tools": {}, "params": {},
            "thresholds": {}, "tool_provenance": {}, "param_provenance": {},
            "assumptions": [], "citations": {}
        }))?,
    )?;
    Ok(())
}

struct ReportTelemetryFixture {
    telemetry: String,
    telemetry_inputs: Vec<(PathBuf, std::fs::Permissions)>,
}

impl ReportTelemetryFixture {
    fn write(outside: &std::path::Path, inside: &std::path::Path) -> Result<Self> {
        let telemetry = [
            ("stage_start", "2024-01-01T00:00:01Z"),
            ("stage_end", "2024-01-01T00:00:02Z"),
        ]
        .into_iter()
        .map(|(event_name, timestamp)| -> Result<String> {
            let event: bijux_dna_runtime::TelemetryEventV1 =
                serde_json::from_value(serde_json::json!({
                    "schema_version": "bijux.telemetry.v1", "run_id": "acquisition-run",
                    "stage_id": "fastq.trim_reads", "tool_id": "fastp", "event_name": event_name,
                    "timestamp": timestamp, "duration_ms": 1, "status": "ok",
                    "trace_id": "trace", "span_id": "span", "attrs": {}, "failure_code": null
                }))?;
            Ok(serde_json::to_string(&event)?)
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n")
            + "\n";
        let mut telemetry_inputs = Vec::new();
        for input_root in [outside, inside] {
            let path = input_root.join("telemetry.jsonl");
            fs::write(&path, &telemetry)?;
            let permissions = fs::metadata(&path)?.permissions();
            telemetry_inputs.push((path, permissions));
        }
        Ok(Self { telemetry, telemetry_inputs })
    }

    fn require_outside_bounds(outside_reproducibility: &serde_json::Value) {
        assert_eq!(outside_reproducibility["started_at"], "2024-01-01T00:00:01+00:00");
        assert_eq!(outside_reproducibility["finished_at"], "2024-01-01T00:00:02+00:00");
    }

    fn verify(self) -> Result<()> {
        let Self { telemetry, telemetry_inputs } = self;
        for (path, permissions) in telemetry_inputs {
            assert_eq!(fs::read(&path)?, telemetry.as_bytes());
            assert_eq!(fs::metadata(&path)?.permissions(), permissions);
        }
        Ok(())
    }
}

#[test]
fn report_acquires_declared_base_inputs_before_presentation_normalization() -> Result<()> {
    use bijux_dna_analyze::report::build_run_report_model;
    let root = fixture_root()?;
    let outside = root.path().join("outside-inputs");
    let outside_base = root.path().join("outside-presentation");
    let inside_base = root.path().join("inside-presentation");
    let inside = inside_base.join(root.path().file_name().expect("private root name"));
    observation_defaults(&outside_base)?;
    observation_defaults(&inside_base)?;
    let (outside_stage, _, _) = base_reports(&outside)?;
    let (inside_stage, _, _) = base_reports(&inside)?;
    let telemetry = ReportTelemetryFixture::write(&outside, &inside)?;
    let outside_model = build_run_report_model(
        &outside_base,
        &[observation_fact(Some(&outside_stage), "acquisition-run")?],
    )?;
    let outside_provenance = serde_json::to_value(&outside_model.sections["metric_provenance"])?;
    assert_eq!(outside_provenance["fastq.trim_reads"]["normalized_params_excerpt"]["min_len"], 20);
    let outside_reproducibility = outside_model.sections["reproducibility"].as_value().clone();
    assert_eq!(outside_model.report.telemetry["event_count"], 2);
    ReportTelemetryFixture::require_outside_bounds(&outside_reproducibility);
    let malformed_path = outside.join("tool_invocation.json");
    let outside_before = fs::read(&malformed_path)?;
    fs::write(&malformed_path, "{not-json")?;
    let malformed = build_run_report_model(
        &outside_base,
        &[observation_fact(Some(&outside_stage), "malformed")?],
    )?;
    assert!(
        malformed.sections["metric_provenance"]
            .as_value()
            .as_object()
            .expect("provenance map")
            .is_empty(),
        "malformed optional input remains an incomplete report"
    );
    fs::write(&malformed_path, outside_before)?;
    let invocation_before = fs::read(inside.join("tool_invocation.json"))?;
    let absolute = build_run_report_model(
        &inside_base,
        &[observation_fact(Some(&inside_stage), "acquisition-run")?],
    )?;
    let relative_stage = inside_stage.strip_prefix(&inside_base)?;
    assert!(
        matches!(std::fs::symlink_metadata(relative_stage), Err(error) if error.kind() == std::io::ErrorKind::NotFound),
        "fixture path must be absent against ambient cwd"
    );
    let relative = build_run_report_model(
        &inside_base,
        &[observation_fact(Some(relative_stage), "acquisition-run")?],
    )?;
    let absolute_provenance = serde_json::to_value(&absolute.sections["metric_provenance"])?;
    let relative_provenance = serde_json::to_value(&relative.sections["metric_provenance"])?;
    assert_eq!(fs::read(inside.join("tool_invocation.json"))?, invocation_before);
    let incomplete =
        build_run_report_model(&inside_base, &[observation_fact(None, "incomplete")?])?;
    assert_eq!(
        incomplete.report.stages.len(),
        1,
        "supported incomplete reporting remains available"
    );
    eprintln!(
        "base_acquisition_observation {}",
        serde_json::json!({
            "outside": outside_provenance, "inside_absolute": absolute_provenance,
            "inside_relative": relative_provenance
        })
    );
    assert_eq!(
        absolute_provenance, outside_provenance,
        "presentation normalization must not change the acquired invocation"
    );
    assert_eq!(
        relative_provenance, outside_provenance,
        "relative report inputs must resolve against the declared base"
    );
    let absolute_reproducibility = absolute.sections["reproducibility"].as_value();
    let relative_reproducibility = relative.sections["reproducibility"].as_value();
    assert_eq!(absolute.report.telemetry["event_count"], 2);
    assert_eq!(relative.report.telemetry["event_count"], 2);
    assert_eq!(
        incomplete.sections["reproducibility"].as_value()["started_at"],
        serde_json::Value::Null
    );
    telemetry.verify()?;
    eprintln!(
        "LATE_TELEMETRY_ACQUISITION_CONTROL_PASS: outside={outside_reproducibility:?}; inside_absolute={absolute_reproducibility:?}; inside_relative={relative_reproducibility:?}; both_event_counts=2; bytes_modes_retained=true"
    );
    for (kind, observed) in
        [("absolute", absolute_reproducibility), ("relative", relative_reproducibility)]
    {
        assert_eq!(
            observed["started_at"], outside_reproducibility["started_at"],
            "presentation normalization must preserve acquired late telemetry start bounds: {kind}"
        );
        assert_eq!(
            observed["finished_at"], outside_reproducibility["finished_at"],
            "presentation normalization must preserve acquired late telemetry finish bounds: {kind}"
        );
    }
    Ok(())
}

fn embedded_report_json(html: &str) -> Result<serde_json::Value> {
    let start = html
        .find("<script id=\"report-json\" type=\"application/json\">")
        .ok_or_else(|| anyhow::anyhow!("report JSON script is absent"))?;
    let body = html[start..].split_once('>').expect("matched complete opening tag").1;
    let raw = body
        .split_once("</script>")
        .ok_or_else(|| anyhow::anyhow!("report JSON closing tag is absent"))?
        .0;
    Ok(serde_json::from_str(raw)?)
}

#[test]
fn html_report_keeps_supported_metadata_as_literal_data() -> Result<()> {
    let root = fixture_root()?;
    let plain_dir = root.path().join("plain");
    let plain_row = observation_fact(None, "plain-run")?;
    write_report_fixture(root.path(), "plain", &[plain_row])?;
    let plain = fs::read_to_string(plain_dir.join("report.html"))?;
    assert_eq!(embedded_report_json(&plain)?["run_id"], "plain-run");
    let literal = "literal </script><span id=\"literal-boundary-sentinel\">data & text</span><script> apostrophe '";
    let mut row = observation_fact(None, literal)?;
    row.tool_version =
        "version <em id=\"version-boundary-sentinel\">literal</em> & \"quoted\"".to_string();
    write_report_fixture(root.path(), "literal", &[row])?;
    let rendered = fs::read_to_string(root.path().join("literal/report.html"))?;
    eprintln!(
        "html_literal_observation {}",
        serde_json::json!({
            "plain_html": plain, "literal_html": rendered,
            "expected_run_id": literal, "expected_stage_id": "fastq.trim_reads",
            "expected_version": "version <em id=\"version-boundary-sentinel\">literal</em> & \"quoted\""
        })
    );
    let parsed = embedded_report_json(&rendered)?;
    assert_eq!(
        parsed["run_id"], literal,
        "a literal script delimiter in supported metadata must not terminate report data"
    );
    assert!(
        !rendered.contains("<em id=\"version-boundary-sentinel\">"),
        "metadata must not become an HTML element"
    );
    Ok(())
}
