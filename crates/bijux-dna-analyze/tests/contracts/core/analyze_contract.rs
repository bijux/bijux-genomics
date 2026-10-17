use bijux_dna_analyze::analyze_contract_v1;

#[test]
fn analyze_contract_has_expected_version() {
    let contract = analyze_contract_v1();
    assert_eq!(contract.schema_version, "bijux.analyze_contract.v1");
    assert!(contract.supported_inputs.contains(&"facts.jsonl".to_string()));
    assert!(contract.outputs.contains(&"report.json".to_string()));
}

fn selection_fact(stage: &str, tool: &str) -> anyhow::Result<bijux_dna_runtime::FactsRowV1> {
    Ok(serde_json::from_value(serde_json::json!({
        "schema_version": "bijux.facts.v1", "run_id": "selection-run",
        "stage_id": stage, "tool_id": tool, "tool_version": "1",
        "image_digest": "sha256:fixture", "trace_id": "trace", "span_id": tool,
        "params_hash": "parameters", "input_hash": "input", "output_hashes": ["output"],
        "runtime_s": 1.0, "memory_mb": 10.0, "exit_code": 0, "bank_hashes": {},
        "reads_in": 100, "reads_out": 100, "bases_in": 1000, "bases_out": 1000,
        "pairs_in": null, "pairs_out": null,
        "metrics": {"reads_in": 100, "reads_out": 100, "bases_in": 1000, "bases_out": 1000},
        "reports": {}, "artifacts": {"metrics_envelope": "metrics_envelope.json"}
    }))?)
}

fn selection_input(
    root: &std::path::Path,
    rows: &[bijux_dna_runtime::FactsRowV1],
    mode: bijux_dna_analyze::AnalyzeMode,
    output_dir: Option<std::path::PathBuf>,
) -> anyhow::Result<bijux_dna_analyze::AnalyzeInput> {
    bijux_dna_infra::ensure_dir(root)?;
    let facts = rows
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n");
    let path = root.join("facts.jsonl");
    bijux_dna_infra::write_bytes(&path, format!("{facts}\n"))?;
    Ok(bijux_dna_analyze::AnalyzeInput {
        run_id: Some("selection-run".to_string()),
        sources: bijux_dna_analyze::AnalyzeSources::FactsJsonl(path),
        options: bijux_dna_analyze::AnalyzeOptions {
            mode,
            strict: false,
            render: bijux_dna_analyze::RenderOptions { json: true, html: false, output_dir },
        },
    })
}

#[test]
fn rank_request_excludes_facts_from_other_stages() -> anyhow::Result<()> {
    use bijux_dna_analyze::{analyze_run, AnalyzeMode};
    let root = bijux_dna_infra::temp_dir("rank-stage-selection")?;
    let mut rows = vec![
        selection_fact("fastq.trim_reads", "selected-a")?,
        selection_fact("fastq.trim_reads", "selected-b")?,
    ];
    let mode = AnalyzeMode::Rank {
        stage_id: bijux_dna_core::ids::StageId::from_static("fastq.trim_reads"),
        metric_id: "runtime_s".parse()?,
    };
    let input = selection_input(root.path(), &rows, mode, None)?;
    let positive = analyze_run(&input)?;
    let positive_path = positive.ranking_json.expect("ranking output");
    let positive_json: serde_json::Value = serde_json::from_slice(&std::fs::read(positive_path)?)?;
    assert!(!positive_json.as_object().expect("ranking modes").is_empty());
    assert!(positive_json
        .as_object()
        .expect("ranking modes")
        .values()
        .all(|entries| entries.as_array().is_some_and(|entries| entries.len() == 2)));
    rows.push(selection_fact("fastq.filter_reads", "foreign-stage-tool")?);
    let selected = selection_input(root.path(), &rows, input.options.mode.clone(), None)?;
    let output = analyze_run(&selected)?;
    let rankings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.ranking_json.expect("ranking output"))?)?;
    let malformed = [selection_fact("unknown.stage", "unknown-tool")?];
    let invalid = selection_input(
        &root.path().join("unknown-input"),
        &malformed,
        input.options.mode.clone(),
        None,
    )?;
    assert!(analyze_run(&invalid)
        .expect_err("unknown source stage")
        .to_string()
        .contains("unknown stage_id"));
    eprintln!("stage_selection_observation {rankings}");
    assert_eq!(
        rankings, positive_json,
        "a requested stage must not acquire unrelated tools or their ranking contributions"
    );
    Ok(())
}

#[test]
fn summary_request_publishes_in_its_selected_output_directory() -> anyhow::Result<()> {
    let root = bijux_dna_infra::temp_dir("summary-output-selection")?;
    let rows = [selection_fact("fastq.trim_reads", "summary-tool")?];
    let default_root = root.path().join("default-output");
    let default_input =
        selection_input(&default_root, &rows, bijux_dna_analyze::AnalyzeMode::Summary, None)?;
    let default_output = bijux_dna_analyze::analyze_run(&default_input)?;
    assert_eq!(default_output.summary_json, Some(default_root.join("run_summary.json")));
    let source = root.path().join("facts-source");
    let destination = root.path().join("selected-output");
    bijux_dna_infra::ensure_dir(&destination)?;
    let input = selection_input(
        &source,
        &rows,
        bijux_dna_analyze::AnalyzeMode::Summary,
        Some(destination.clone()),
    )?;
    let facts_before = std::fs::read(source.join("facts.jsonl"))?;
    let output = bijux_dna_analyze::analyze_run(&input)?;
    assert_eq!(std::fs::read(source.join("facts.jsonl"))?, facts_before);
    eprintln!("summary_output_observation {output:?}");
    assert_eq!(
        output.summary_json,
        Some(destination.join("run_summary.json")),
        "explicit render output authority must select the published summary root"
    );
    assert!(!source.join("run_summary.json").exists());
    assert!(destination.join("run_summary.json").is_file());
    assert!(!destination.join("report.html").exists());
    Ok(())
}

#[test]
fn report_request_preserves_json_without_publishing_disabled_html() -> anyhow::Result<()> {
    let root = bijux_dna_infra::temp_dir("report-format-selection")?;
    let rows = [selection_fact("fastq.trim_reads", "report-tool")?];
    for (name, html) in [("both-formats", true), ("json-only", false)] {
        let directory = root.path().join(name);
        let mut input =
            selection_input(&directory, &rows, bijux_dna_analyze::AnalyzeMode::Report, None)?;
        input.options.render.html = html;
        bijux_dna_infra::write_bytes(
            directory.join("defaults_ledger.json"),
            serde_json::to_vec(&serde_json::json!({
                "pipeline_id": "fastq-to-fastq__default__v1", "tools": {}, "params": {},
                "thresholds": {}, "tool_provenance": {}, "param_provenance": {},
                "assumptions": [], "citations": {}
            }))?,
        )?;
        let output = bijux_dna_analyze::analyze_run(&input)?;
        assert_eq!(output.report_json, Some(directory.join("report.json")));
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("report.json"))?)?;
        assert_eq!(report["run_id"], "selection-run");
        eprintln!(
            "report_format_observation {name}: html_exists={}",
            directory.join("report.html").exists()
        );
        assert_eq!(
            directory.join("report.html").exists(),
            html,
            "rendered HTML must follow the explicit format selection"
        );
    }
    Ok(())
}
