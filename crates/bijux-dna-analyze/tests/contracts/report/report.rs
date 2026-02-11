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
        warnings: vec!["low_q".to_string()],
        errors: vec![],
        invariants: vec![],
        verdict: Some(StageVerdictV1 {
            stage_id: "fastq.trim".to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: Vec::new(),
            key_metrics: serde_json::json!({}),
        }),
        outputs: vec!["out.fastq.gz".to_string()],
        subreports: vec![],
        log_paths: vec![],
    };
    bijux_dna_infra::write_bytes(
        &stage_report_path,
        serde_json::to_vec_pretty(&stage_report)?,
    )?;

    let effective_config = EffectiveConfigV1 {
        schema_version: "bijux.effective_config.v1".to_string(),
        stage_id: "fastq.trim".to_string(),
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
        stage_id: bijux_dna_core::ids::StageId::from_static("fastq.trim"),
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
        stage_id: "fastq.trim".to_string(),
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
        "stage_id": "fastq.trim",
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
        stage_id: "fastq.trim".to_string(),
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
        stage_id: "fastq.trim".to_string(),
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
    let Some(sections) = report_value
        .get("sections")
        .and_then(|value| value.as_object())
    else {
        panic!("sections block missing")
    };
    for key in [
        "qc",
        "trimming",
        "filtering",
        "contamination",
        "retention",
        "failures",
    ] {
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
        stage_id: "fastq.merge".to_string(),
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
        stage_id: "fastq.validate_pre".to_string(),
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
        stage_id: "fastq.trim".to_string(),
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
        assert!(!entry["tool_version"]
            .as_str()
            .unwrap_or_default()
            .is_empty());
        assert!(!entry["params_hash"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["image_digest"]
            .as_str()
            .unwrap_or_default()
            .is_empty());
        assert!(!entry["trace_id"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["span_id"].as_str().unwrap_or_default().is_empty());
        assert!(!entry["bank_hashes"].is_null());
    }
    Ok(())
}
