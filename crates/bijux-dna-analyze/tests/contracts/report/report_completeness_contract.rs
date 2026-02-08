use anyhow::Result;
use bijux_dna_analyze::report::write_run_report_from_facts;
use bijux_dna_runtime::FactsRowV1;

#[test]
#[allow(clippy::too_many_lines)]
fn report_completeness_policy_requires_provenance_and_contracts() -> Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-report-policy")?;
    let dir = temp.path();

    let facts = vec![FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:img".to_string()),
        trace_id: "trace".to_string(),
        span_id: "span".to_string(),
        params_hash: "params".to_string(),
        input_hash: "input".to_string(),
        output_hashes: vec!["out".to_string()],
        runtime_s: 1.0,
        memory_mb: 1.0,
        exit_code: 0,
        bank_hashes: serde_json::Value::default(),
        reads_in: Some(100),
        reads_out: Some(90),
        bases_in: Some(1000),
        bases_out: Some(900),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({"reads_in": 100}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }];

    let facts_path = dir.join("facts.jsonl");
    let mut facts_raw = String::new();
    for row in &facts {
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

    let run_manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "run_id": "run-1",
        "pipeline_id": "fastq-to-fastq__default__v1",
        "profile_id": "default",
        "graph_hash": "sha256:graph",
        "dataset_fingerprints": ["sha256:input"],
        "stage_contracts": {
            "fastq.trim": "sha256:contract"
        }
    });
    bijux_dna_infra::write_bytes(
        dir.join("run_manifest.json"),
        serde_json::to_vec_pretty(&run_manifest)?,
    )?;

    let report_path = write_run_report_from_facts(dir, &facts)?;
    let report_raw = std::fs::read_to_string(report_path)?;
    let report: serde_json::Value = serde_json::from_str(&report_raw)?;

    let stages = report
        .get("stages")
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("report stages array missing or invalid"));
    for stage in stages {
        assert!(stage
            .get("tool_version")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty()));
        assert!(stage
            .get("params_hash")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty()));
        assert!(stage
            .get("input_hash")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty()));
    }

    let run_provenance = report
        .get("sections")
        .and_then(|value| value.get("run_provenance"))
        .unwrap_or_else(|| panic!("run_provenance section missing"));
    assert!(run_provenance
        .get("graph_hash")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value != "unknown"));
    assert!(run_provenance
        .get("input_hashes")
        .and_then(|value| value.as_array())
        .is_some_and(|value| !value.is_empty()));
    assert!(run_provenance
        .get("stage_contracts")
        .and_then(|value| value.as_object())
        .is_some_and(|value| !value.is_empty()));

    let analysis_contract = report
        .get("sections")
        .and_then(|value| value.get("analysis_selection_contract"))
        .unwrap_or_else(|| panic!("analysis selection contract missing"));
    assert!(analysis_contract.get("objectives").is_some());

    Ok(())
}
