#[test]
#[allow(clippy::too_many_lines)]
fn fastq_stages_emit_observability_contracts() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let image = test_image();
    for stage in bijux_stages_fastq::fastq::registry() {
        if stage.id == "fastq.preprocess" {
            continue;
        }
        let out_dir = dir.path().join(stage.id.replace('.', "_"));
        bijux_infra::ensure_dir(&out_dir).context("create out dir")?;
        let (exec_plan, outputs, is_retention) = build_plan(stage.id, &r1, &r2, &out_dir, &image)?;
        for output in &outputs {
            touch(output).context("touch output")?;
        }
        if stage.id == "fastq.qc_post" {
            touch(&out_dir.join("fastqc_trimmed").join("fastqc_data.txt"))
                .context("touch fastqc output")?;
            touch(&out_dir.join("multiqc_report.html")).context("touch multiqc output")?;
            bijux_infra::ensure_dir(out_dir.join("multiqc_data")).context("touch multiqc data")?;
        }
        if stage.id == "fastq.merge" {
            for output in merge_outputs_for(&exec_plan.tool_id.0, &out_dir) {
                touch(&output).context("touch merge output")?;
            }
        }

        let _result = execute_plan(&exec_plan, RunnerKind::Docker, None)?;
        let run_artifacts = out_dir.join("run_artifacts");
        assert!(run_artifacts.join("plan.json").exists());
        assert!(run_artifacts.join("effective_config.json").exists());
        assert!(run_artifacts.join("metrics_envelope.json").exists());
        assert!(run_artifacts.join("stage_report.json").exists());
        assert!(run_artifacts.join("stage_metrics.json").exists());
        let stage_metrics_raw = fs::read_to_string(run_artifacts.join("stage_metrics.json"))?;
        let stage_metrics: serde_json::Value = serde_json::from_str(&stage_metrics_raw)?;
        let metrics = stage_metrics["metrics"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("stage_metrics.metrics missing"))?;
        if matches!(
            stage.id,
            "fastq.trim"
                | "fastq.filter"
                | "fastq.merge"
                | "fastq.correct"
                | "fastq.umi"
                | "fastq.preprocess"
        ) {
            assert!(metrics.contains_key("reads_in"));
            assert!(metrics.contains_key("reads_out"));
            assert!(metrics.contains_key("bases_in"));
            assert!(metrics.contains_key("bases_out"));
        }
        let invocation_path = run_artifacts
            .join("invocations")
            .join(format!("{}.tool_invocation.json", stage.id));
        assert!(invocation_path.exists());
        assert!(run_artifacts.join("stage_events.jsonl").exists());
        let telemetry_dir = run_artifacts.join("telemetry");
        assert!(telemetry_dir.join("events.jsonl").exists());
        assert!(telemetry_dir.join("timings.json").exists());
        assert!(telemetry_dir.join("resources.json").exists());
        assert!(telemetry_dir.join("errors.json").exists());
        let stage_config = run_artifacts
            .join("config")
            .join(format!("{}.effective.json", stage.id));
        assert!(stage_config.exists());
        let effective_config_path = run_artifacts.join("effective_config.json");
        assert!(effective_config_path.exists());
        let effective_config_raw = fs::read_to_string(&effective_config_path)?;
        let effective_config: serde_json::Value = serde_json::from_str(&effective_config_raw)?;
        assert_eq!(effective_config["stage_id"], stage.id);
        assert_eq!(
            effective_config["tool_id"],
            serde_json::Value::from(exec_plan.tool_id.0.clone())
        );
        assert_eq!(
            effective_config["tool_version"],
            serde_json::Value::from(exec_plan.tool_version.clone())
        );
        assert!(effective_config.get("image_digest").is_some());
        assert!(effective_config.get("runner").is_some());
        assert!(effective_config.get("platform").is_some());
        assert!(effective_config.get("resources").is_some());
        assert!(effective_config.get("parameters_json").is_some());
        assert!(effective_config.get("parameters_json_normalized").is_some());
        assert!(effective_config.get("effective_params_json").is_some());
        assert!(effective_config
            .get("effective_params_json_normalized")
            .is_some());
        if stage.id == "fastq.trim" {
            assert!(effective_config.get("adapter_bank").is_some());
            assert!(effective_config.get("bank_assets").is_some());
        }
        if stage.affects_read_counts || is_retention {
            let retention_path = run_artifacts
                .join("reports")
                .join(format!("{}.retention.json", stage.id));
            assert!(retention_path.exists());
            let retention_raw = fs::read_to_string(&retention_path)?;
            let retention: serde_json::Value = serde_json::from_str(&retention_raw)?;
            assert!(
                retention
                    .get("retention")
                    .and_then(|v| v.get("definition"))
                    .and_then(|v| v.as_str())
                    .is_some(),
                "retention definition missing"
            );
            assert!(
                retention
                    .get("condition")
                    .and_then(|v| v.get("banks"))
                    .is_some(),
                "retention condition banks missing"
            );
            assert!(
                retention
                    .get("condition")
                    .and_then(|v| v.get("parameters"))
                    .is_some(),
                "retention condition parameters missing"
            );
            for key in [
                "min_len",
                "q",
                "merge_policy",
                "adapter_policy",
                "polyx_policy",
                "contaminant_policy",
            ] {
                assert!(
                    retention
                        .get("condition")
                        .and_then(|v| v.get(key))
                        .is_some(),
                    "retention condition {key} missing"
                );
            }
        }
        let manifest_path = run_artifacts.join("observability_manifest.json");
        assert!(manifest_path.exists());
        let manifest_raw = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)?;
        let artifacts = manifest["artifacts"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("manifest artifacts missing"))?;
        let mut names = std::collections::BTreeSet::new();
        for artifact in artifacts {
            if let Some(name) = artifact["name"].as_str() {
                names.insert(name.to_string());
            }
        }
        assert!(names.contains("plan"));
        assert!(names.contains("effective_config"));
        assert!(names.contains("stage_config"));
        assert!(names.contains("tool_invocation"));
        assert!(names.contains("metrics_envelope"));
        assert!(names.contains("stage_metrics"));
        assert!(names.contains("stage_report"));
        if stage.affects_read_counts {
            assert!(names.contains("retention_report"));
        }

        let report_path = run_artifacts.join("stage_report.json");
        let report_raw = fs::read_to_string(&report_path)?;
        let report: serde_json::Value = serde_json::from_str(&report_raw)?;
        assert!(report.get("metrics_path").is_some());
        assert!(report.get("tool_invocation_path").is_some());
        assert!(report.get("effective_config_path").is_some());
        assert!(report.get("facts_row_id").is_some());
        let report_metrics_path = report["metrics_path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("report metrics_path missing"))?;
        assert!(Path::new(report_metrics_path).exists());
        let report_invocation_path = report["tool_invocation_path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("report tool_invocation_path missing"))?;
        assert!(Path::new(report_invocation_path).exists());

        let envelope_path = run_artifacts.join("metrics_envelope.json");
        let envelope_raw = fs::read_to_string(&envelope_path).context("read metrics envelope")?;
        let envelope: serde_json::Value = serde_json::from_str(&envelope_raw)?;
        assert_eq!(envelope["stage_id"], stage.id);
        assert_eq!(
            envelope["stage_version"],
            serde_json::Value::from(stage_version_i32(stage.version))
        );
        assert_eq!(envelope["tool_id"], exec_plan.tool_id.0);
        assert_eq!(envelope["tool_version"], exec_plan.tool_version);
        assert!(
            envelope["input_hash"].as_str().is_some(),
            "input_hash missing"
        );
        let canonical_params = parameters_json_canonicalization(&exec_plan.params);
        let expected_hash = params_hash(&canonical_params)?;
        assert_eq!(envelope["params_hash"], expected_hash);
        assert_eq!(envelope["parameters_json"], canonical_params);

        let progress_path = run_artifacts.join("progress.jsonl");
        assert!(progress_path.exists());
        let progress_raw = fs::read_to_string(&progress_path)?;
        let progress_line = progress_raw
            .lines()
            .last()
            .ok_or_else(|| anyhow::anyhow!("missing progress event"))?;
        let progress: serde_json::Value = serde_json::from_str(progress_line)?;
        assert_eq!(progress["stage_id"], stage.id);
        assert_eq!(progress["tool_id"], exec_plan.tool_id.0);

        let runs_path = run_artifacts.join("dashboard").join("runs.jsonl");
        assert!(runs_path.exists());
        let runs_raw = fs::read_to_string(&runs_path)?;
        let runs_line = runs_raw
            .lines()
            .last()
            .ok_or_else(|| anyhow::anyhow!("missing runs export"))?;
        let run_row: serde_json::Value = serde_json::from_str(runs_line)?;
        assert_eq!(run_row["stage_id"], stage.id);
        assert_eq!(run_row["tool_id"], exec_plan.tool_id.0);

        let telemetry_path = run_artifacts.join("telemetry").join("events.jsonl");
        assert!(telemetry_path.exists());
        let telemetry_raw = fs::read_to_string(&telemetry_path)?;
        let mut has_stage_start = false;
        let mut has_stage_end = false;
        let mut has_tool_start = false;
        let mut has_tool_end = false;
        let mut has_artifact_written = false;
        for line in telemetry_raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let event: bijux_core::TelemetryEventV1 = serde_json::from_str(line)?;
            assert_eq!(event.schema_version, "bijux.telemetry.v1");
            assert!(!event.run_id.is_empty());
            assert_eq!(event.stage_id, stage.id);
            assert_eq!(event.tool_id, exec_plan.tool_id.0);
            assert!(!event.trace_id.is_empty());
            assert!(!event.span_id.is_empty());
            assert!(!event.timestamp.is_empty());
            if event.event_name == "stage_start" {
                has_stage_start = true;
            }
            if event.event_name == "stage_end" {
                has_stage_end = true;
            }
            if event.event_name == "tool_start" {
                has_tool_start = true;
            }
            if event.event_name == "tool_end" {
                has_tool_end = true;
            }
            if event.event_name == "artifact_written" {
                has_artifact_written = true;
            }
        }
        assert!(has_stage_start, "missing stage_start telemetry event");
        assert!(has_stage_end, "missing stage_end telemetry event");
        assert!(has_tool_start, "missing tool_start telemetry event");
        assert!(has_tool_end, "missing tool_end telemetry event");
        assert!(
            has_artifact_written,
            "missing artifact_written telemetry event"
        );

        let facts_path = run_artifacts.join("dashboard").join("facts.jsonl");
        assert!(facts_path.exists());
        let facts_payload = fs::read_to_string(&facts_path)?;
        let facts_lines: Vec<&str> = facts_payload
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert_eq!(facts_lines.len(), 1);
        let facts_line = facts_lines
            .last()
            .ok_or_else(|| anyhow::anyhow!("missing facts rows"))?;
        let facts_record: bijux_core::FactsRowV1 = serde_json::from_str(facts_line)?;
        assert_eq!(facts_record.schema_version, "bijux.facts.v1");
        assert_eq!(facts_record.stage_id, stage.id);
        assert_eq!(facts_record.tool_id, exec_plan.tool_id.0);
        assert!(!facts_record.params_hash.is_empty());
        assert!(facts_record.bank_hashes.is_object());

        let invocation_raw = fs::read_to_string(&invocation_path)?;
        let invocation: bijux_core::ToolInvocationV1 = serde_json::from_str(&invocation_raw)?;
        assert_eq!(invocation.stage_id, exec_plan.stage_id.0);
        assert_eq!(invocation.tool_id, exec_plan.tool_id.0);
        assert!(!invocation.tool_version.is_empty());
        assert!(!invocation.runner_kind.is_empty());
        assert!(!invocation.image_digest.is_empty());
        assert!(!invocation.platform.is_empty());
        assert!(!invocation.input_hashes.is_empty());
    }

    std::env::set_var("PATH", original_path);
    Ok(())
}

#[test]
fn preprocess_emits_telemetry_events() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, _) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let old_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), old_path));

    let pipeline = bijux_planner_fastq::default_pipeline_spec(
        bijux_planner_fastq::DefaultPipelineOptions {
            paired: false,
            ..Default::default()
        },
    );
    let preprocess_plan = bijux_stages_fastq::fastq::preprocess::PreprocessPlan {
        r1: r1.clone(),
        r2: None,
        pipeline,
        merge_decision: None,
        correct_decision: None,
        enable_contaminant_removal: false,
    };
    let tool = dummy_tool("fastp", &test_image());
    let plan =
        bijux_stages_fastq::fastq::preprocess::plan_preprocess_stage(&preprocess_plan, &tool);
    let _result = execute_plan(&plan, RunnerKind::Docker, None)?;

    let run_artifacts = bijux_exec::run_artifacts::run_artifacts_dir_for_out(&plan.out_dir);
    let telemetry_path = run_artifacts.join("telemetry").join("events.jsonl");
    assert!(telemetry_path.exists());
    let telemetry_raw = fs::read_to_string(&telemetry_path)?;
    let events: Vec<bijux_core::TelemetryEventV1> = telemetry_raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    assert!(events.len() >= 4, "expected at least 4 telemetry events");
    assert!(events.iter().any(|e| e.event_name == "stage_start"));
    assert!(events.iter().any(|e| e.event_name == "stage_end"));
    assert!(events.iter().any(|e| e.event_name == "tool_start"));
    assert!(events.iter().any(|e| e.event_name == "tool_end"));

    std::env::set_var("PATH", old_path);
    Ok(())
}
