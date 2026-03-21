/// Run the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn bench_fastq_preprocess<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    fastq_preprocess_run(catalog, platform, runner_override, args)
}

/// Execute the preprocess pipeline.
///
/// # Errors
/// Returns an error if planning or execution fails.
pub fn fastq_preprocess_run<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<()> {
    let normalized_sample_id = canonical_sample_identity(&args.sample_id);
    let bench_dir_name = bench_dir_name(&STAGE_PREPROCESS_SUMMARY).ok_or_else(|| {
        anyhow!(
            "bench dir missing for {}",
            STAGE_PREPROCESS_SUMMARY.as_str()
        )
    })?;
    let out_dir = bench_base_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&out_dir).context("create preprocess output dir")?;
    let run_root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
    bijux_dna_infra::ensure_dir(&run_root).context("create preprocess run dir")?;
    let entry_invariants = write_fastq_entry_invariants(&run_root, &args.r1, args.r2.as_deref())?;
    maybe_write_fastq_coverage_classifier(&run_root, &entry_invariants)?;
    let primer_governance = enforce_primer_governance(&run_root, args, &entry_invariants)?;
    write_reference_db_validation_artifact(&run_root, None, primer_governance.as_ref())?;
    write_contamination_controls_report(&run_root, &normalized_sample_id)?;
    write_batch_effect_summary(&run_root, &normalized_sample_id, &entry_invariants)?;
    if args.r2.is_some() && !entry_invariants.paired_consistent {
        return Err(anyhow!(
            "fastq entry invariants failed: {}",
            entry_invariants
                .paired_reason
                .unwrap_or_else(|| "paired-end consistency failed".to_string())
        ));
    }

    ensure_bench_runner(platform, runner_override)?;

    let registry =
        load_workspace_registry().map_err(|err| anyhow!("manifest validation failed: {err}"))?;
    let decisions = preprocess_decisions(args);
    let pipeline = resolve_preprocess_pipeline(args, &decisions);
    if args.mode == bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::Shotgun {
        let amplicon_only = [
            "fastq.normalize_primers",
            "fastq.remove_chimeras",
            "fastq.infer_asvs",
            "fastq.cluster_otus",
            "fastq.normalize_abundance",
        ];
        if let Some(stage) = pipeline
            .stage_catalog()
            .iter()
            .find(|stage| amplicon_only.contains(&stage.as_str()))
        {
            return Err(anyhow!(
                "stage {stage} is not applicable in shotgun mode; use --mode edna_amplicon or --mode pollen_amplicon"
            ));
        }
    }
    let bench_repo = if args.auto {
        Some(SqliteBenchResultsRepository::new(args.out.clone()))
    } else {
        None
    };
    if args.auto && args.run_all_governed_tools {
        return Err(anyhow!(
            "--auto and --run-all-governed-tools cannot be combined; automatic selection chooses one tool per stage while governed fan-out expands all admitted runtime tools"
        ));
    }
    let jobs = bench_jobs(args.jobs);
    let runtime_pipeline = pipeline.clone();
    let mut planner_stage_toolsets = Vec::new();
    let mut selected_stage_tools = if args.run_all_governed_tools {
        let toolsets = bijux_dna_planner_fastq::select_preprocess_toolsets(
            &runtime_pipeline,
            bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
            args.allow_planned,
        )?;
        let filtered_toolsets = toolsets
            .into_iter()
            .map(|toolset| {
                let filtered = filter_tools_by_role(
                    &toolset.stage_id,
                    &toolset.tool_ids,
                    &registry,
                    false,
                )?;
                Ok(bijux_dna_planner_fastq::ToolsetSelection {
                    stage_id: toolset.stage_id,
                    stage_instance_id: toolset.stage_instance_id,
                    tool_ids: filtered,
                    reason: toolset.reason,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        planner_stage_toolsets = filtered_toolsets
            .iter()
            .map(|toolset| {
                let tools = toolset
                    .tool_ids
                    .iter()
                    .map(|tool_id| {
                        let spec = build_tool_execution_spec(
                            toolset.stage_id.as_str(),
                            tool_id.as_str(),
                            &registry,
                            catalog,
                            platform,
                        )?;
                        Ok(scale_tool_spec_for_jobs(&spec, jobs))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(bijux_dna_planner_fastq::FastqStageToolsetBinding {
                    stage_id: toolset.stage_id.clone(),
                    stage_instance_id: toolset.stage_instance_id.clone(),
                    tools,
                    reason: Some(toolset.reason.clone()),
                    params: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let (_expanded_pipeline, expanded_stage_tools) =
            bijux_dna_planner_fastq::expand_pipeline_stage_tool_routes(
                &runtime_pipeline,
                &filtered_toolsets,
            )?;
        expanded_stage_tools
    } else {
        select_preprocess_stage_tools(
            &registry,
            &runtime_pipeline,
            args,
            bench_repo
                .as_ref()
                .map(|repo| repo as &dyn bijux_dna_planner_fastq::BenchResultsRepository),
        )?
    };
    let mut filtered_stage_tools = Vec::new();
    for selection in &selected_stage_tools {
        let mut allowed =
            filter_tools_by_role(&selection.stage_id, std::slice::from_ref(&selection.tool_id), &registry, false)?;
        if let Some(selected) = allowed.pop() {
            filtered_stage_tools.push(StageToolSelection {
                stage_id: selection.stage_id.clone(),
                stage_instance_id: selection.stage_instance_id.clone(),
                tool_id: selected,
                reason: selection.reason.clone(),
            });
        }
    }
    selected_stage_tools = filtered_stage_tools;
    let tool_ids: Vec<String> = selected_stage_tools
        .iter()
        .map(|selection| selection.tool_id.clone())
        .collect();

    write_explain_plan_json(
        &out_dir,
        STAGE_PREPROCESS_SUMMARY.as_str(),
        &tool_ids,
        &registry,
        None,
    )?;

    ensure_image_qa_passed(
        STAGE_PREPROCESS_SUMMARY.as_str(),
        &tool_ids,
        platform,
        catalog,
    )?;
    ensure_tool_qa_passed(
        STAGE_PREPROCESS_SUMMARY.as_str(),
        &tool_ids,
        platform,
        catalog,
    )?;
    let tools_root = bench_tools_dir(&args.out, bench_dir_name, &args.sample_id);
    bijux_dna_infra::ensure_dir(&tools_root).context("create preprocess tools dir")?;

    let policy = apply_preprocess_policy(
        selected_stage_tools
            .iter()
            .map(|selection| StageId::new(selection.stage_id.clone()))
            .collect(),
        selected_stage_tools
            .iter()
            .map(|selection| ToolId::new(selection.tool_id.clone()))
            .collect(),
    );

    let adapter_bank = adapter_bank_context(
        policy
            .adapter_bank_preset_override
            .as_deref()
            .or(args.adapter_bank_preset.as_deref()),
        args.adapter_bank.as_deref(),
        args.adapter_bank_file.as_deref(),
        &args.enable_adapters,
        &args.disable_adapters,
    )?;
    let polyx_bank = polyx_bank_context(args.polyx_preset.as_deref())?;
    let contaminant_bank = contaminant_bank_context(args.contaminant_preset.as_deref())?;

    let mut failures = Vec::new();
    let mut tool_specs = Vec::new();
    for selection in &selected_stage_tools {
        let stage = StageId::new(selection.stage_id.clone());
        let tool = ToolId::new(selection.tool_id.clone());
        let spec =
            build_tool_execution_spec(stage.as_str(), tool.as_str(), &registry, catalog, platform)?;
        let spec = scale_tool_spec_for_jobs(&spec, jobs);
        if stage == STAGE_TRIM_READS {
            if let Some(msg) = polyx_unsupported_warning(
                &spec.tool_id.0,
                polyx_bank.as_ref(),
                args.polyx_preset.is_some(),
            ) {
                eprintln!("{msg}");
            }
        }
        tool_specs.push(spec);
    }
    let mut aux_tools = std::collections::BTreeMap::new();
    if policy
        .pipeline_stages
        .iter()
        .any(|stage| stage == &STAGE_REPORT_QC)
    {
        for aux_tool in bijux_dna_planner_fastq::stage_api::fastq::report_qc::aux_tool_ids() {
            let spec = catalog
                .get(aux_tool.as_str())
                .ok_or_else(|| anyhow!("tool {aux_tool} missing from images.toml"))?;
            let image = resolve_image_for_run(spec, platform)?;
            aux_tools.insert(
                aux_tool,
                ContainerImageRefV1 {
                    image: image.full_name,
                    digest: spec.digest.clone(),
                },
            );
        }
    }
    let pipeline_id = args
        .profile
        .as_deref()
        .unwrap_or("fastq-to-fastq__default__v1")
        .to_string();
    let planner_config = FastqPlanConfig {
        pipeline_id,
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(runtime_pipeline.clone()),
        stage_bindings: if planner_stage_toolsets.is_empty() {
            selected_stage_tools
                .iter()
                .zip(tool_specs.iter())
                .map(|(selection, tool)| FastqStageBinding {
                    stage_id: selection.stage_id.clone(),
                    stage_instance_id: selection.stage_instance_id.clone(),
                    tool: tool.clone(),
                    reason: Some(selection.reason.clone()),
                    params: None,
                })
                .collect()
        } else {
            Vec::new()
        },
        stage_toolsets: planner_stage_toolsets,
        stages: if args.run_all_governed_tools {
            Vec::new()
        } else {
            policy
                .pipeline_stages
                .iter()
                .map(|stage| stage.as_str().to_string())
                .collect()
        },
        tools: if args.run_all_governed_tools {
            Vec::new()
        } else {
            tool_specs.clone()
        },
        aux_images: aux_tools.clone(),
        adapter_bank: adapter_bank.clone(),
        polyx_bank: polyx_bank.clone(),
        contaminant_bank: contaminant_bank.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        reference_fasta: args.reference_fasta.clone(),
        out_dir: bench_tools_dir(&args.out, bench_dir_name, &args.sample_id),
        tool_reasons: None,
        allow_planned: args.allow_planned,
    };
    let pipeline_plan = FastqPlanner::plan(&planner_config)?;
    let planned_stage_batches = execution_step_batches(&pipeline_plan)?;
    let planned_stages = planned_stage_batches
        .iter()
        .flat_map(|batch| batch.iter().cloned())
        .collect::<Vec<_>>();
    if matches!(
        args.mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) && planned_stages.iter().any(|step| {
        step.step_id.as_str().starts_with("bam.") || step.step_id.as_str().starts_with("vcf.")
    }) {
        return Err(anyhow!(
            "amplicon mode refusal: BAM/VCF stages are not schedulable in eDNA/pollen preprocess pipeline"
        ));
    }
    let required_tools = required_fastq_tools()?;
    for planned in &planned_stages {
        let stage_id = planned.step_id.to_string();
        let tool_id = planned
            .command
            .template
            .first()
            .map(String::as_str)
            .unwrap_or_default();
        enforce_stage_applicability(planned, args, contaminant_bank.as_ref())?;
        enforce_fastq_backend_allowlist(&stage_id, tool_id)?;
        if !required_tools.contains(tool_id) {
            return Err(anyhow!(
                "tool `{tool_id}` for stage `{stage_id}` is not declared in configs/ci/tools/required_tools.toml"
            ));
        }
        enforce_screen_db_governance(planned)?;
    }
    std::env::set_var(
        "BIJUX_PLANNER_VERSION",
        bijux_dna_planner_fastq::PLANNER_VERSION,
    );

    if args.dry_run {
        let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
        bijux_dna_infra::ensure_dir(&root).context("create dry-run artifacts dir")?;
        let mut stage_runs: Vec<StageExecutionSummary> = planned_stages
            .iter()
            .map(|plan| StageExecutionSummary {
                plan: plan.clone(),
                result: StageResultV1 {
                    run_id: "dry-run".to_string(),
                    exit_code: 0,
                    runtime_s: 0.0,
                    memory_mb: 0.0,
                    outputs: plan
                        .io
                        .outputs
                        .iter()
                        .map(|artifact| artifact.path.clone())
                        .collect(),
                    metrics_path: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    command: "dry-run".to_string(),
                },
            })
            .collect();
        let report_plan = report_stage_step(&args.out, &planned_stages);
        let mut steps = planned_stages.clone();
        steps.push(report_plan.clone());
        let mut edges = pipeline_plan.edges().to_vec();
        for terminal in terminal_step_ids(&pipeline_plan) {
            edges.push(ExecutionEdge::new(terminal, report_plan.step_id.clone()));
        }
        let graph = ExecutionGraph::new(
            pipeline_plan.pipeline_id().to_string(),
            pipeline_plan.planner_version().to_string(),
            pipeline_plan.policy(),
            steps,
            edges,
        )?;
        let graph_path = root.join("graph.json");
        bijux_dna_infra::atomic_write_json(&graph_path, &graph).context("write graph.json")?;
        stage_runs.push(StageExecutionSummary {
            plan: report_plan,
            result: StageResultV1 {
                run_id: "dry-run".to_string(),
                exit_code: 0,
                runtime_s: 0.0,
                memory_mb: 0.0,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "dry-run".to_string(),
            },
        });
        let decision_trace = serde_json::json!({
            "schema_version": "bijux.decision_trace.v1",
            "stage": STAGE_PREPROCESS_SUMMARY.as_str(),
            "selections": selected_stage_tools
                .iter()
                .map(|selection| serde_json::json!({
                    "stage_id": selection.stage_id,
                    "stage_instance_id": selection.stage_instance_id,
                    "tool_id": selection.tool_id,
                    "reason": selection.reason,
                }))
                .collect::<Vec<_>>(),
            "merge_decision": decisions.merge_decision.as_ref(),
            "correct_decision": decisions.correct_decision.as_ref(),
            "adapter_inference": policy.adapter_inference.as_ref(),
            "stage_skips": &policy.stage_skips,
        });
        bijux_dna_infra::atomic_write_json(&root.join("decision_trace.json"), &decision_trace)
            .context("write decision_trace.json")?;
        let artifact_manifest = serde_json::json!({
            "schema_version": "bijux.plan_artifacts.v1",
            "pipeline_id": pipeline_plan.pipeline_id(),
            "artifacts": planned_stages
                .iter()
                .map(|plan| serde_json::json!({
                    "stage_id": plan.step_id.to_string(),
                    "image": plan.image.image,
                    "outputs": plan
                        .io
                        .outputs
                        .iter()
                        .map(|artifact| serde_json::json!({
                            "name": artifact.name,
                            "kind": artifact.role.as_str(),
                            "path": artifact.path,
                        }))
                        .collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>(),
        });
        bijux_dna_infra::atomic_write_json(
            &root.join("plan_artifact_manifest.json"),
            &artifact_manifest,
        )
        .context("write plan_artifact_manifest.json")?;
        write_run_manifest(&args.out, &stage_runs, &failures)?;
        write_scientific_provenance(&args.out, &stage_runs)?;
        return Ok(());
    }

    let telemetry = build_telemetry_adapter();
    let mut pipeline_attrs = std::collections::BTreeMap::new();
    pipeline_attrs.insert("sample_id".to_string(), normalized_sample_id.clone());
    pipeline_attrs.insert(
        "pipeline".to_string(),
        STAGE_PREPROCESS_SUMMARY.as_str().to_string(),
    );
    let pipeline_span =
        telemetry.start_pipeline(STAGE_PREPROCESS_SUMMARY.as_str(), &pipeline_attrs);

    let mut stage_runs = Vec::new();
    let mut fail_fast_triggered = false;
    for batch in planned_stage_batches {
        if fail_fast_triggered {
            break;
        }
        let batch_results = execute_preprocess_batch(
            &batch,
            platform.runner,
            jobs,
            &out_dir,
            &normalized_sample_id,
            args,
        )?;
        for (planned, batch_result) in batch.into_iter().zip(batch_results.into_iter()) {
            let stage_id = planned.step_id.to_string();
            let tool = planned.image.image.clone();
            let stage_root = run_artifacts_dir_for_out(&out_dir).join(planned.step_id.as_str());
            let resume_hit = batch_result.stdout == "resumed" && batch_result.command == "resume";
            let mut stage_attrs = std::collections::BTreeMap::new();
            stage_attrs.insert("stage".to_string(), stage_id.clone());
            stage_attrs.insert("tool".to_string(), tool.clone());
            let stage_span = telemetry.start_stage(&stage_id, &stage_attrs);
            capture_tool_version(
                &stage_root,
                planned
                    .command
                    .template
                    .first()
                    .map_or("unknown", String::as_str),
            )?;
            write_stage_standardized_metrics(
                &stage_root,
                &stage_id,
                &planned.out_dir,
                &batch_result,
            )?;
            emit_fastq_stage_extra_artifacts(&stage_root, &stage_id, &batch_result)?;
            write_stage_governance_artifacts(&stage_root, &planned, contaminant_bank.as_ref())?;
            enforce_metrics_schema(&stage_root, &stage_id)?;
            write_fastq_output_contract(&stage_root, &planned, &batch_result)?;
            write_stage_resume_contract(&stage_root, &stage_id, &batch_result, resume_hit)?;
            if matches!(
                stage_id.as_str(),
                "fastq.trim_terminal_damage"
                    | "fastq.normalize_primers"
                    | "fastq.remove_chimeras"
                    | "fastq.cluster_otus"
                    | "fastq.infer_asvs"
                    | "fastq.normalize_abundance"
            ) {
                let stage_metrics = materialize_amplicon_stage_outputs(&stage_root, &planned)?;
                enforce_amplicon_qc_thresholds(&stage_root, &stage_id, &stage_metrics)?;
            }
            if stage_id == "fastq.merge_pairs" {
                write_merge_join_contract(
                    &stage_root,
                    &batch_result,
                    entry_invariants.paired_consistent,
                )?;
                enforce_amplicon_merge_determinism(&stage_root, args.mode, &batch_result)?;
            }
            write_retention_report(&stage_root, &planned)?;
            if batch_result.exit_code != 0 {
                let hint =
                    classify_failure_hint(&stage_id, &batch_result.stdout, &batch_result.stderr);
                let hint_path = stage_root.join("common_failure_hint.json");
                bijux_dna_infra::atomic_write_json(
                    &hint_path,
                    &serde_json::json!({
                        "schema_version": "bijux.failure_hint.v1",
                        "stage_id": stage_id,
                        "hint": hint,
                        "exit_code": batch_result.exit_code,
                    }),
                )?;
                if stage_id == "fastq.validate_reads" {
                    return Err(anyhow!(
                        "strict validation failed in fastq.validate_reads; refusing pipeline execution"
                    ));
                }
                failures.push(RawFailure {
                    stage: stage_id,
                    tool: tool.clone(),
                    reason: format!(
                        "tool failed with status {}. hint: {}",
                        batch_result.exit_code, hint
                    ),
                    category: ErrorCategory::ToolError,
                });
                fail_fast_triggered = true;
            }
            stage_span.end();
            stage_runs.push(StageExecutionSummary {
                plan: planned,
                result: batch_result,
            });
        }
    }
    pipeline_span.end();

    render_run_summary(
        &args.out,
        &stage_runs,
        &failures,
        decisions.merge_decision.as_ref(),
        decisions.correct_decision.as_ref(),
        policy.adapter_inference.as_ref(),
        &policy.stage_skips,
    )?;
    let executed_steps: Vec<_> = stage_runs.iter().map(|entry| entry.plan.clone()).collect();
    let report_plan = report_stage_step(&args.out, &executed_steps);
    let report_outputs = report_plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let report_run_id = stage_runs.first().map_or_else(
        || "report.aggregate".to_string(),
        |entry| entry.result.run_id.clone(),
    );
    let report_result = StageResultV1 {
        run_id: report_run_id,
        exit_code: 0,
        runtime_s: 0.0,
        memory_mb: 0.0,
        outputs: report_outputs,
        metrics_path: None,
        stdout: String::new(),
        stderr: String::new(),
        command: "report-aggregate".to_string(),
    };
    stage_runs.push(StageExecutionSummary {
        plan: report_plan,
        result: report_result,
    });
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&out_dir);
    bijux_dna_infra::ensure_dir(&root).context("create run artifacts dir")?;
    write_retry_policy(&root)?;
    write_taxonomy_db_drift_report(&root, contaminant_bank.as_ref())?;
    write_reference_db_validation_artifact(
        &root,
        contaminant_bank.as_ref(),
        primer_governance.as_ref(),
    )?;
    let decision_trace_path = root.join("decision_trace.json");
    let identity_norm = serde_json::json!({
        "schema_version": "bijux.identity_normalization.v1",
        "sample_id_raw": args.sample_id.clone(),
        "sample_id_normalized": normalized_sample_id,
        "lane_id": "L001",
    });
    bijux_dna_infra::atomic_write_json(&root.join("identity_normalization.json"), &identity_norm)
        .context("write identity_normalization.json")?;
    let decision_trace = serde_json::json!({
        "schema_version": "bijux.decision_trace.v1",
        "stage": STAGE_PREPROCESS_SUMMARY.as_str(),
        "merge_decision": decisions.merge_decision.as_ref(),
        "correct_decision": decisions.correct_decision.as_ref(),
        "adapter_inference": policy.adapter_inference.as_ref(),
        "stage_skips": &policy.stage_skips,
    });
    bijux_dna_infra::atomic_write_json(&decision_trace_path, &decision_trace)
        .context("write decision_trace.json")?;
    let steps: Vec<_> = stage_runs.iter().map(|entry| entry.plan.clone()).collect();
    let mut edges = pipeline_plan.edges().to_vec();
    if let Some(report) = steps.last() {
        for terminal in terminal_step_ids(&pipeline_plan) {
            if terminal != report.step_id {
                edges.push(ExecutionEdge::new(terminal, report.step_id.clone()));
            }
        }
    }
    let graph = ExecutionGraph::new(
        pipeline_plan.pipeline_id().to_string(),
        pipeline_plan.planner_version().to_string(),
        pipeline_plan.policy(),
        steps,
        edges,
    )?;
    let graph_path = root.join("graph.json");
    bijux_dna_infra::atomic_write_json(&graph_path, &graph).context("write graph.json")?;
    write_run_manifest(&args.out, &stage_runs, &failures)?;
    write_scientific_provenance(&args.out, &stage_runs)?;
    write_edna_report_summary(&root, &stage_runs)?;
    if let Some(decision) = decisions.merge_decision.as_ref() {
        let run_id = stage_runs
            .first()
            .map(|entry| entry.result.run_id.clone())
            .unwrap_or_default();
        let telemetry_path = run_artifacts_dir_for_out(&out_dir)
            .join("telemetry")
            .join("events.jsonl");
        let event = TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id,
            stage_id: STAGE_PREPROCESS_SUMMARY.as_str().to_string(),
            tool_id: "planner".to_string(),
            event_name: TelemetryEventName::MergeDecision,
            timestamp: chrono::Utc::now(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: "merge-decision".to_string(),
            span_id: "merge-decision".to_string(),
            attrs: attrs_from_json(
                &serde_json::to_value(decision).unwrap_or_else(|_| serde_json::json!({})),
            ),
            failure_code: None,
        };
        let _ = write_telemetry_event(&telemetry_path, &event);
    }
    if !failures.is_empty() {
        return Err(anyhow!(
            "preprocess pipeline failed: {} failures",
            failures.len()
        ));
    }

    Ok(())
}

fn execution_step_batches(graph: &ExecutionGraph) -> Result<Vec<Vec<ExecutionStep>>> {
    let mut incoming = std::collections::BTreeMap::<String, usize>::new();
    let mut outgoing = std::collections::BTreeMap::<String, Vec<String>>::new();
    for step in graph.steps() {
        incoming.insert(step.step_id.as_str().to_string(), 0);
    }
    for edge in graph.edges() {
        *incoming.entry(edge.to().as_str().to_string()).or_insert(0) += 1;
        outgoing
            .entry(edge.from().as_str().to_string())
            .or_default()
            .push(edge.to().as_str().to_string());
    }
    let mut ready = incoming
        .iter()
        .filter_map(|(node_id, count)| if *count == 0 { Some(node_id.clone()) } else { None })
        .collect::<Vec<_>>();
    ready.sort();
    let mut batches = Vec::new();
    let mut visited = 0usize;
    while !ready.is_empty() {
        let current_batch_ids = std::mem::take(&mut ready);
        let mut batch = current_batch_ids
            .iter()
            .map(|step_id| {
                graph.step_by_id(step_id).cloned().ok_or_else(|| {
                    anyhow!(
                        "execution graph is missing planned step {} during runtime batching",
                        step_id
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        batch.sort_by(|left, right| left.step_id.as_str().cmp(right.step_id.as_str()));
        visited += batch.len();
        batches.push(batch);
        let mut next_ready = Vec::new();
        for node_id in current_batch_ids {
            if let Some(children) = outgoing.get(&node_id) {
                for child in children {
                    if let Some(count) = incoming.get_mut(child) {
                        *count -= 1;
                        if *count == 0 {
                            next_ready.push(child.clone());
                        }
                    }
                }
            }
        }
        next_ready.sort();
        next_ready.dedup();
        ready = next_ready;
    }
    if visited != graph.steps().len() {
        return Err(anyhow!(
            "execution graph batching did not visit all steps; graph may be cyclic"
        ));
    }
    Ok(batches)
}

fn terminal_step_ids(graph: &ExecutionGraph) -> Vec<bijux_dna_core::prelude::StepId> {
    let mut outgoing = std::collections::BTreeSet::new();
    for edge in graph.edges() {
        outgoing.insert(edge.from().as_str().to_string());
    }
    graph
        .steps()
        .iter()
        .filter(|step| !outgoing.contains(step.step_id.as_str()))
        .map(|step| step.step_id.clone())
        .collect()
}

fn execute_preprocess_batch(
    batch: &[ExecutionStep],
    runner: RuntimeKind,
    jobs: usize,
    out_dir: &std::path::Path,
    normalized_sample_id: &str,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<Vec<StageResultV1>> {
    let mut resumable = Vec::new();
    let mut pending = Vec::new();
    for (idx, planned) in batch.iter().enumerate() {
        let stage_id = planned.step_id.to_string();
        let stage_root = run_artifacts_dir_for_out(out_dir).join(planned.step_id.as_str());
        write_stage_path_contract(&stage_root, &stage_id, planned, args.r2.is_some())?;
        let expected_outputs = planned
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>();
        let runtime_marker = stage_root.join("runtime_provenance.json");
        let resume_hit = runtime_marker.exists() && expected_outputs.iter().all(|path| path.exists());
        if resume_hit {
            resumable.push((
                idx,
                StageResultV1 {
                    run_id: format!("fastq-preprocess-{}", planned.step_id),
                    exit_code: 0,
                    runtime_s: 0.0,
                    memory_mb: 0.0,
                    outputs: expected_outputs,
                    metrics_path: None,
                    stdout: "resumed".to_string(),
                    stderr: String::new(),
                    command: "resume".to_string(),
                },
            ));
            continue;
        }
        pending.push((
            idx,
            execution_kernel::ToolInvocationRequest {
                step: planned.clone(),
                runner,
                context: execution_kernel::ToolContext {
                    run_id: format!("fastq-preprocess-{}", planned.step_id),
                    stage_id: planned.step_id.to_string(),
                    tool_id: planned.image.image.clone(),
                    sample_id: Some(normalized_sample_id.to_string()),
                    stage_root: stage_root.clone(),
                    input_root: args
                        .r1
                        .parent()
                        .map_or_else(|| out_dir.to_path_buf(), std::path::Path::to_path_buf),
                    output_root: out_dir.to_path_buf(),
                    tmp_root: stage_root.join("tmp"),
                    threads: 1,
                    memory_hint_mb: None,
                    compression_threads: Some(1),
                    seed: None,
                    network_policy: stage_network_policy(&stage_id),
                },
                timeout: None,
                mode: execution_kernel::ToolExecMode::Execute,
            },
        ));
    }
    let executed = if jobs <= 1 || pending.len() <= 1 {
        pending
            .iter()
            .map(|(_, request)| execution_kernel::ToolExec::invoke(request).map(|result| result.stage_result))
            .collect::<Result<Vec<_>>>()?
    } else {
        let total = pending.len();
        let queue = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::from(
            pending.iter().cloned().collect::<Vec<_>>(),
        )));
        let results: std::sync::Arc<std::sync::Mutex<Vec<Option<Result<StageResultV1>>>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(total)));
        {
            let mut guard = results
                .lock()
                .map_err(|_| anyhow!("preprocess batch results lock poisoned"))?;
            guard.resize_with(total, || None);
        }
        let job_count = jobs.min(total);
        let mut workers = Vec::new();
        for _ in 0..job_count {
            let queue = std::sync::Arc::clone(&queue);
            let results = std::sync::Arc::clone(&results);
            workers.push(std::thread::spawn(move || loop {
                let next = {
                    match queue.lock() {
                        Ok(mut guard) => guard.pop_front(),
                        Err(_) => None,
                    }
                };
                let Some((slot, request)) = next else {
                    break;
                };
                let value = execution_kernel::ToolExec::invoke(&request).map(|result| result.stage_result);
                if let Ok(mut guard) = results.lock() {
                    guard[slot] = Some(value);
                } else {
                    break;
                }
            }));
        }
        for worker in workers {
            let _ = worker.join();
        }
        let results = {
            let mut guard = results
                .lock()
                .map_err(|_| anyhow!("preprocess batch results lock poisoned"))?;
            std::mem::take(&mut *guard)
        };
        let mut out = Vec::with_capacity(results.len());
        for entry in results {
            let value = entry.unwrap_or_else(|| Err(anyhow!("preprocess batch execution result missing")))?;
            out.push(value);
        }
        out
    };
    let mut results = vec![None; batch.len()];
    for (idx, result) in resumable {
        results[idx] = Some(result);
    }
    for ((idx, _), result) in pending.into_iter().zip(executed.into_iter()) {
        results[idx] = Some(result);
    }
    results
        .into_iter()
        .map(|result| result.ok_or_else(|| anyhow!("missing batch execution result")))
        .collect()
}

#[cfg(test)]
mod pipeline_run_tests {
    use super::{execution_step_batches, terminal_step_ids};
    use anyhow::Result;
    use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, PlanPolicy, StageIO, ToolConstraints};
    use bijux_dna_core::prelude::{ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

    fn step(id: &str) -> bijux_dna_core::contract::ExecutionStep {
        bijux_dna_core::contract::ExecutionStep {
            step_id: StepId::new(id.to_string()),
            stage_id: StageId::new("fastq.trim_reads".to_string()),
            command: CommandSpecV1 {
                template: vec!["echo".to_string(), id.to_string()],
            },
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1".to_string()),
                    std::path::PathBuf::from("reads_R1.fastq"),
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("report_json".to_string()),
                    std::path::PathBuf::from(format!("{id}.json")),
                    ArtifactRole::SummaryJson,
                )],
            },
            out_dir: std::path::PathBuf::from(id),
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: vec![ArtifactId::new("report_json".to_string())],
            metrics_schema_ids: Vec::new(),
        }
    }

    #[test]
    fn execution_step_batches_group_independent_roots_together() -> Result<()> {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__runtime_batches__v1",
            "planner.test",
            PlanPolicy::default(),
            vec![step("a"), step("b"), step("c")],
            vec![
                ExecutionEdge::new(StepId::new("a".to_string()), StepId::new("c".to_string())),
                ExecutionEdge::new(StepId::new("b".to_string()), StepId::new("c".to_string())),
            ],
        )?;

        let batches = execution_step_batches(&graph)?;
        assert_eq!(batches.len(), 2);
        assert_eq!(
            batches[0]
                .iter()
                .map(|step| step.step_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(
            batches[1]
                .iter()
                .map(|step| step.step_id.as_str())
                .collect::<Vec<_>>(),
            vec!["c"]
        );
        Ok(())
    }

    #[test]
    fn terminal_step_ids_ignore_non_terminal_compare_parents() -> Result<()> {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__terminal_steps__v1",
            "planner.test",
            PlanPolicy::default(),
            vec![step("trim.fastp"), step("trim.cutadapt"), step("trim.compare")],
            vec![
                ExecutionEdge::new(
                    StepId::new("trim.fastp".to_string()),
                    StepId::new("trim.compare".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim.cutadapt".to_string()),
                    StepId::new("trim.compare".to_string()),
                ),
            ],
        )?;
        let terminals = terminal_step_ids(&graph);
        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0].as_str(), "trim.compare");
        Ok(())
    }

    #[test]
    fn execution_step_batches_keep_select_rejoin_after_branch_candidates() -> Result<()> {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__select_rejoin_batches__v1",
            "planner.test",
            PlanPolicy::default(),
            vec![
                step("trim.fastp"),
                step("trim.cutadapt"),
                step("trim.select"),
                step("filter.selected"),
            ],
            vec![
                ExecutionEdge::new(
                    StepId::new("trim.fastp".to_string()),
                    StepId::new("trim.select".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim.cutadapt".to_string()),
                    StepId::new("trim.select".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim.select".to_string()),
                    StepId::new("filter.selected".to_string()),
                ),
            ],
        )?;

        let batches = execution_step_batches(&graph)?;
        assert_eq!(batches.len(), 3);
        assert_eq!(
            batches[0]
                .iter()
                .map(|step| step.step_id.as_str())
                .collect::<Vec<_>>(),
            vec!["trim.cutadapt", "trim.fastp"]
        );
        assert_eq!(
            batches[1]
                .iter()
                .map(|step| step.step_id.as_str())
                .collect::<Vec<_>>(),
            vec!["trim.select"]
        );
        assert_eq!(
            batches[2]
                .iter()
                .map(|step| step.step_id.as_str())
                .collect::<Vec<_>>(),
            vec!["filter.selected"]
        );
        Ok(())
    }
}
