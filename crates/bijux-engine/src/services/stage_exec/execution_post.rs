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
    input_paths: &[PathBuf],
    input_hashes: &[String],
    input_hash: &str,
    metric_context: &MetricContextV1,
    plan_artifacts: &crate::services::run_artifacts::PlanArtifacts,
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
    let stage_metrics = if plan.stage_id.0 == "fastq.filter" {
        let removals =
            filter_removals_for_plan(plan.tool_id.0.as_str(), &plan.out_dir, canonical_params);
        filter_metrics_with_removals(
            plan.stage_id.0.as_str(),
            input_paths,
            &outputs,
            canonical_params,
            &plan.effective_params,
            &removals,
        )?
    } else {
        stage_metrics_for_plan(
            plan.stage_id.0.as_str(),
            input_paths,
            &outputs,
            canonical_params,
            &plan.effective_params,
        )?
    };
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
    let metrics_path = run_artifacts_dir.join("metrics.json");
    let facts_row_id = format!("{}:{}:{}", run_id, plan.stage_id.0, plan.tool_id.0);
    let reports = build_stage_reports_and_warnings(
        plan,
        run_id,
        run_artifacts_dir,
        trace_id,
        span_id,
        canonical_params,
        adapter_bank,
        bank_assets,
        banks_json,
        input_paths,
        &outputs,
        &stage_metrics,
        &metrics_path,
        &tool_invocation_path,
        plan_artifacts,
        &facts_row_id,
        &log_paths,
        params_hash,
        input_hash,
        &output_hashes,
        &metrics_envelope_path,
        &stage_metrics_path,
        emit_event,
        emit_artifact,
    )?;
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
        canonical_params,
        params_hash,
        input_hash,
        &output_hashes,
        runtime_s,
        memory_mb,
        &execution,
        &stage_metrics,
        &metrics_envelope_path,
        &reports.stage_report_path,
        reports.retention_report_path.as_deref(),
        reports.effective_adapters_path.as_deref(),
        &log_paths,
        reports.quality_gate.as_ref(),
        reports.adapter_validation.as_ref(),
        reports.contaminant_action,
        &reports.assertion_results,
        &reports.invariant_verdict,
        &reports.subreports,
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in,
        pairs_out,
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
