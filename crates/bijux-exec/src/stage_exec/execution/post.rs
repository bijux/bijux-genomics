struct ExecutionPostResult {
    stage_result: StageResultV1,
    output_hashes: Vec<String>,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn process_execution_result(
    plan: &StagePlanV1,
    run_id: &str,
    sample_id: &str,
    run_artifacts_dir: &Path,
    trace_id: &str,
    span_id: &str,
    started_at: chrono::DateTime<Utc>,
    canonical_params: &serde_json::Value,
    params_hash: &str,
    adapter_bank: Option<&AdapterBankProvenanceV1>,
    banks_json: Option<&serde_json::Value>,
    bank_assets: Option<&serde_json::Value>,
    input_hashes: &[String],
    input_hash: &str,
    metric_context: &MetricContextV1,
    plan_artifacts: &bijux_engine::services::run_artifacts::PlanArtifacts,
    image_digest: &str,
    runner_kind: &str,
    execution: ExecutionEnvelope,
    outputs: Vec<PathBuf>,
    runtime_s: f64,
    memory_mb: f64,
    emit_event: &dyn Fn(&bijux_core::TelemetryEventV1) -> Result<()>,
    emit_artifact: &dyn Fn(&str, &Path) -> Result<()>,
) -> Result<ExecutionPostResult> {
    let output_hashes = hash_outputs(&outputs)?;
    let log_paths = write_execution_logs_bounded(
        &run_artifacts_dir.join("logs"),
        &execution.stdout,
        &execution.stderr,
    )?;
    for path in &log_paths {
        emit_artifact("execution_log", path)?;
    }
    let plugin_output = if let Some(plugin) = crate::plugins::select_stage_plugin(&plan.stage_id.0)
    {
        let output_refs = plan.io.outputs.clone();
        plugin.parse_outputs(plan, &output_refs)?
    } else {
        bijux_core::stage_plugin::StagePluginOutputV1 {
            metrics: serde_json::json!({}),
            artifacts: Vec::new(),
            report_parts: Vec::new(),
            warnings: Vec::new(),
            invariants: Vec::new(),
            verdict: None,
            event_hints: Vec::new(),
        }
    };
    let stage_metrics = plugin_output.metrics.clone();
    let invocation = ToolInvocationV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        tool_version: plan.tool_version.clone(),
        resolved_tool_version: Some(plan.tool_version.clone()),
        image_digest: image_digest.to_string(),
        runner_kind: runner_kind.to_string(),
        platform: std::env::var("BIJUX_PLATFORM").unwrap_or_else(|_| "unknown".to_string()),
        parameters_json: canonical_params.clone(),
        parameters_json_normalized: bijux_core::parameters_json_canonicalization(canonical_params),
        effective_params_json: plan.effective_params.clone(),
        effective_params_json_normalized: bijux_core::parameters_json_canonicalization(
            &plan.effective_params,
        ),
        adapter_bank: adapter_bank.cloned(),
        banks: banks_json.cloned(),
        bank_assets: bank_assets.cloned(),
        resources: plan.resources.clone(),
        environment: std::env::vars().collect::<BTreeMap<String, String>>(),
        input_hashes: input_hashes.to_vec(),
        output_hashes: output_hashes.clone(),
        executed_command: Some(execution.command.clone()),
    };
    let tool_invocation_path =
        write_tool_invocation_json(run_artifacts_dir, &plan.stage_id.0, &invocation)?;
    emit_artifact("tool_invocation", &tool_invocation_path)?;
    let ctx = StageObservabilityContextV1 {
        stage_id: plan.stage_id.0.clone(),
        stage_version: stage_version_i32(plan.stage_version),
        tool_id: plan.tool_id.0.clone(),
        tool_version: plan.tool_version.clone(),
        input_hash: input_hash.to_string(),
        params_hash: params_hash.to_string(),
        parameters_json: canonical_params.clone(),
        metric_context: metric_context.clone(),
    };
    let execution_metrics = bijux_core::measure::ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let metrics_envelope_path = write_metrics_envelope(
        run_artifacts_dir,
        &ctx,
        &execution_metrics,
        &stage_metrics,
        &output_hashes,
    )?;
    emit_artifact("metrics_envelope", &metrics_envelope_path)?;
    let stage_metrics_payload = StageMetricsV1 {
        schema_version: "bijux.stage_metrics.v1".to_string(),
        stage_id: plan.stage_id.0.clone(),
        stage_version: stage_version_i32(plan.stage_version),
        tool_id: plan.tool_id.0.clone(),
        tool_version: plan.tool_version.clone(),
        context: metric_context.clone(),
        execution: execution_metrics,
        failure_class: None,
        failure_reason: None,
        metrics: stage_metrics.clone(),
    };
    let stage_metrics_path = write_stage_metrics_json(run_artifacts_dir, &stage_metrics_payload)?;
    emit_artifact("stage_metrics", &stage_metrics_path)?;
    let mut report_parts_map = serde_json::Map::new();
    if !plugin_output.report_parts.is_empty() {
        let reports_dir = run_artifacts_dir.join("reports");
        bijux_infra::ensure_dir(&reports_dir).context("create reports dir")?;
        for part in &plugin_output.report_parts {
            let path = reports_dir.join(&part.file_name);
            bijux_infra::atomic_write_json(&path, &part.payload)
                .with_context(|| format!("write report part {}", part.name))?;
            emit_artifact(&part.name, &path)?;
            report_parts_map.insert(part.name.clone(), serde_json::json!(path.display().to_string()));
        }
    }
    for hint in &plugin_output.event_hints {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: hint.event_name.clone(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: hint.status.clone(),
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            attrs: hint.attrs.clone(),
        })?;
    }
    for warning in &plugin_output.warnings {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "warn".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "warn".to_string(),
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            attrs: serde_json::json!({
                "message": warning,
            }),
        })?;
    }
    let mut plugin_artifacts_map = serde_json::Map::new();
    let mut extra_manifest_artifacts = Vec::new();
    for artifact in &plugin_output.artifacts {
        emit_artifact(&artifact.name, &artifact.path)?;
        plugin_artifacts_map.insert(
            artifact.name.clone(),
            serde_json::json!(artifact.path.display().to_string()),
        );
        extra_manifest_artifacts.push(serde_json::json!({
            "name": format!("artifact:{}", artifact.name),
            "path": artifact.path,
        }));
    }
    for (name, path) in &report_parts_map {
        extra_manifest_artifacts.push(serde_json::json!({
            "name": format!("report_part:{name}"),
            "path": path,
        }));
    }
    if !log_paths.is_empty() {
        extra_manifest_artifacts.push(serde_json::json!({
            "name": "execution_logs",
            "paths": log_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>(),
        }));
    }
    let (reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out) =
        extract_io_deltas(&stage_metrics);
    let reads_in = reads_in.unwrap_or(0);
    let reads_out = reads_out.unwrap_or(0);
    let bases_in = bases_in.unwrap_or(0);
    let bases_out = bases_out.unwrap_or(0);
    let pairs_in = pairs_in.unwrap_or(0);
    let pairs_out = pairs_out.unwrap_or(0);

    write_facts_and_records(
        plan,
        run_id,
        run_artifacts_dir,
        trace_id,
        span_id,
        params_hash,
        input_hash,
        &output_hashes,
        runtime_s,
        memory_mb,
        &execution,
        &stage_metrics,
        &metrics_envelope_path,
        &log_paths,
        &serde_json::Value::Object(report_parts_map),
        &serde_json::Value::Object(plugin_artifacts_map),
        &plugin_output.warnings,
        &plugin_output.invariants,
        plugin_output.verdict.as_ref(),
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
    )?;
    let _observability_manifest = write_observability_manifest(
        run_artifacts_dir,
        &plan.stage_id.0,
        &plan.tool_id.0,
        &plan_artifacts.plan_path,
        &plan_artifacts.effective_config_path,
        &plan_artifacts.stage_config_path,
        &tool_invocation_path,
        &metrics_envelope_path,
        &stage_metrics_path,
        None,
        None,
        &extra_manifest_artifacts,
    )?;
    write_progress_and_runs(
        plan,
        run_id,
        run_artifacts_dir,
        started_at,
        runtime_s,
        memory_mb,
        &execution,
        params_hash,
        input_hash,
        &metrics_envelope_path,
        &outputs,
    )?;
    let marker_path = plan.out_dir.join("engine_execution.json");
    let marker = serde_json::json!({
        "schema_version": "bijux.engine_execution.v1",
        "stage": plan.stage_id.0,
        "tool": plan.tool_id.0,
    });
    bijux_infra::atomic_write_json(&marker_path, &marker).context("write engine execution marker")?;
    let stage_result = StageResultV1 {
        run_id: run_id.to_string(),
        exit_code: execution.exit_code,
        runtime_s,
        memory_mb,
        outputs,
        metrics_path: Some(metrics_envelope_path),
        stdout: execution.stdout,
        stderr: execution.stderr,
        command: execution.command,
    };
    info!(
        run_id = %run_id,
        sample_id = %sample_id,
        stage = %plan.stage_id.0,
        tool = %plan.tool_id.0,
        tool_version = %plan.tool_version,
        image_digest = %image_digest,
        params_hash = %params_hash,
        input_hash = %input_hash,
        exit_code = execution.exit_code,
        runtime_s = runtime_s,
        memory_mb = memory_mb,
        "stage execution finished"
    );
    Ok(ExecutionPostResult {
        stage_result,
        output_hashes,
    })
}
