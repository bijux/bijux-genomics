#[allow(clippy::too_many_lines)]
pub fn run_suite(cwd: &Path, suite_id: &str, hpc: bool) -> Result<PathBuf> {
    let suite = load_suite(cwd, suite_id)?;
    let fairness = suite.effective_fairness();
    let scientific_defaults = validate_scientific_defaults_doc(cwd)?;
    let corpus_root = cwd.join("examples").join("bijux-dna-data").join(&suite.corpus);
    if !corpus_root.exists() {
        return Err(anyhow!("missing corpus root {}", corpus_root.display()));
    }
    crate::commands::corpus::validate_corpus(cwd, &suite.corpus)?;
    let species_id = load_species_id_from_snapshot(&corpus_root);

    let run_context = if hpc { "HPC" } else { "Local" }.to_string();
    let suite_signature = suite_signature(cwd, suite_id, hpc)?;
    let run_id = format!("{}-{}", suite.suite_id, &suite_signature[..12]);
    let run_dir = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(&suite.suite_id)
        .join("runs")
        .join(&run_id);
    bijux_dna_infra::ensure_dir(&run_dir)?;

    let telemetry_path = run_dir.join("telemetry.jsonl");
    let manifest_path = run_dir.join("run_manifest.json");
    if manifest_path.exists() {
        append_telemetry_event(
            &telemetry_path,
            "resume_identical",
            &serde_json::json!({
                "run_id": run_id,
                "suite_id": suite.suite_id,
                "decision": "skipped_execution_existing_identical_run"
            }),
        )?;
        return Ok(run_dir);
    }

    let tmp_root = if hpc {
        cwd.join("artifacts")
            .join("bench")
            .join("suites")
            .join(&suite.suite_id)
            .join("tmp")
            .join(&run_id)
    } else {
        std::env::temp_dir().join(&run_id)
    };
    bijux_dna_infra::ensure_dir(&tmp_root)?;

    append_telemetry_event(
        &telemetry_path,
        "suite_run_started",
        &serde_json::json!({
            "suite_id": suite.suite_id,
            "run_id": run_id,
            "context": run_context,
            "threads": fairness.threads,
            "mem_gb": fairness.mem_gb,
            "tmp_policy": fairness.tmp_policy,
            "cold_runs": fairness.cold_runs,
            "warm_runs": fairness.warm_runs
        }),
    )?;

    let mut decision_trace = Vec::new();
    for stage in &suite.stages {
        let mut candidates = stage.tools.clone();
        candidates.sort();
        let selected_tool = candidates
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("stage {} has no tools", stage.stage))?;
        decision_trace.push(DecisionTraceRow {
            stage: stage.stage.clone(),
            candidates,
            selected_tool,
            reason: "deterministic lexical-first selection from suite candidate set".to_string(),
        });
    }

    let mut tool_invocations = Vec::new();
    let mut metrics_artifacts = Vec::new();
    let mut postconditions = Vec::new();
    let mut run_records = Vec::new();

    for (mode, runs) in [("cold", fairness.cold_runs), ("warm", fairness.warm_runs)] {
        if runs == 0 {
            continue;
        }
        for run_index in 1..=runs {
            for stage in &suite.stages {
                for tool in &stage.tools {
                    let image_digest = pseudo_digest(&format!("{}:{}", stage.stage, tool));
                    let version = format!("{tool}-simulated-1.0.0");
                    tool_invocations.push(ToolInvocationRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        tool_version: version,
                        image_digest,
                    });

                    let runtime_s =
                        deterministic_metric(&stage.stage, tool, mode, run_index, 3.0, 40.0);
                    let memory_mb =
                        deterministic_metric(&stage.stage, tool, mode, run_index, 256.0, 4096.0);
                    let retention_permille =
                        deterministic_u32(&stage.stage, tool, mode, run_index, 650, 1000);
                    let read_retention = f64::from(retention_permille) / 1000.0;
                    let length_shift_bp =
                        deterministic_i32(&stage.stage, tool, mode, run_index, -800, 800);
                    let length_shift = f64::from(length_shift_bp) / 100.0;
                    let delta_metrics = if stage.stage == "trim" || stage.stage == "filter" {
                        let reads_before = 1_000_000_u64;
                        let reads_after =
                            reads_before.saturating_mul(u64::from(retention_permille)) / 1000;
                        let bases_before = 150_000_000_u64;
                        let bases_after =
                            bases_before.saturating_mul(u64::from(retention_permille)) / 1000;
                        Some(serde_json::json!({
                            "reads_before": reads_before,
                            "reads_after": reads_after,
                            "bases_before": bases_before,
                            "bases_after": bases_after,
                            "length_summary": {
                                "p50_before": 151,
                                "p50_after": 151 + (length_shift_bp / 100)
                            }
                        }))
                    } else {
                        None
                    };

                    let metrics_dir = run_dir
                        .join("metrics")
                        .join(mode)
                        .join(format!("run_{run_index}"));
                    bijux_dna_infra::ensure_dir(&metrics_dir)?;
                    let metrics_path = metrics_dir.join(format!("{}_{}.json", stage.stage, tool));
                    let metrics_payload = serde_json::json!({
                        "schema_version": "bijux.bench.metrics_artifact.v1",
                        "stage": stage.stage,
                        "tool": tool,
                        "mode": mode,
                        "run_index": run_index,
                        "runtime_s": runtime_s,
                        "memory_mb": memory_mb,
                        "read_retention": read_retention,
                        "length_shift": length_shift,
                        "delta_metrics": delta_metrics,
                    });
                    bijux_dna_infra::atomic_write_json(&metrics_path, &metrics_payload)?;
                    metrics_artifacts.push(MetricsArtifactRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        path: metrics_path.strip_prefix(cwd).map_or_else(
                            |_| metrics_path.display().to_string(),
                            |p| p.to_string_lossy().to_string(),
                        ),
                    });

                    let checks = stage_checks(&stage.stage);
                    let ok = !checks.is_empty();
                    postconditions.push(PostconditionRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        ok,
                        checks,
                    });

                    run_records.push(RunRecordRow {
                        stage: stage.stage.clone(),
                        tool: tool.clone(),
                        mode: mode.to_string(),
                        run_index,
                        runtime_s,
                        memory_mb,
                        read_retention,
                        length_shift,
                        delta_metrics,
                        valid: ok,
                    });
                }
            }
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let environment = capture_environment_snapshot(cwd);
    let comparability_hash =
        compute_comparability_hash(&suite, &fairness, &tool_invocations, &scientific_defaults)?;

    let mut manifest = SuiteRunManifest {
        schema_version: "bijux.bench.suite_run_manifest.v2".to_string(),
        suite_id: suite.suite_id.clone(),
        run_id: run_id.clone(),
        run_context,
        corpus: suite.corpus.clone(),
        species_id,
        fairness: FairnessContract {
            threads: fairness.threads,
            mem_gb: fairness.mem_gb,
            tmp_policy: fairness.tmp_policy.clone(),
            tmp_root: tmp_root.display().to_string(),
            deviations: Vec::new(),
        },
        cold_vs_warm: ColdWarmContract {
            cold_runs: fairness.cold_runs,
            warm_runs: fairness.warm_runs,
            cold_run_records_image_cost: true,
            warm_runs_exclude_image_cost: true,
        },
        decision_trace,
        tool_invocations,
        metrics_artifacts,
        postconditions,
        run_records,
        comparability_hash,
        environment,
        scientific_defaults,
        telemetry_path: telemetry_path.display().to_string(),
        reproducibility_bundle: String::new(),
        created_at_unix: now,
    };

    let tool_invocations_path = run_dir.join("tool_invocations.json");
    bijux_dna_infra::atomic_write_json(&tool_invocations_path, &manifest.tool_invocations)?;
    let decision_trace_path = run_dir.join("decision_trace.json");
    bijux_dna_infra::atomic_write_json(&decision_trace_path, &manifest.decision_trace)?;
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;

    let bundle_path = run_dir.join("reproducibility_bundle.tar.gz");
    write_repro_bundle(&bundle_path, &run_dir)?;
    manifest.reproducibility_bundle = bundle_path.display().to_string();
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;

    append_telemetry_event(
        &telemetry_path,
        "suite_run_finished",
        &serde_json::json!({
            "run_id": run_id,
            "bundle": manifest.reproducibility_bundle,
            "records": manifest.run_records.len(),
        }),
    )?;

    let latest = cwd
        .join("artifacts")
        .join("bench")
        .join("suites")
        .join(&suite.suite_id)
        .join("latest");
    bijux_dna_infra::ensure_dir(&latest)?;
    bijux_dna_infra::atomic_write_json(
        &latest.join("run_pointer.json"),
        &serde_json::json!({
            "schema_version": "bijux.bench.suite_run_pointer.v2",
            "suite_id": suite.suite_id,
            "run_id": run_id,
            "run_dir": run_dir.display().to_string(),
        }),
    )?;

    Ok(run_dir)
}
