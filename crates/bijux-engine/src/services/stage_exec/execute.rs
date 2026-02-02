#[allow(clippy::too_many_lines)]
pub fn execute_stage_plan(
    plan: &StagePlanV1,
    runner: RunnerKind,
    mut observer: Option<&mut dyn Observer>,
) -> Result<StageResultV1> {
    let run_id = Uuid::new_v4().to_string();
    let (r1, r2) = match plan.io.inputs.as_slice() {
        [] => (None, None),
        [r1] => (Some(r1.path.as_path()), None),
        [r1, r2, ..] => (Some(r1.path.as_path()), Some(r2.path.as_path())),
    };
    let r1 = r1.ok_or_else(|| anyhow!("plan inputs missing r1"))?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("input r1 has no parent directory"))?;
    let container_name = format!("bijux-stage-{}-{}", plan.stage_id.0, Uuid::new_v4());
    let run_artifacts_dir = run_artifacts_dir_for_out(&plan.out_dir);
    std::fs::create_dir_all(&run_artifacts_dir).context("create run_artifacts dir")?;
    let (trace_id, span_id) = default_trace_ids();
    let telemetry_path = std::env::var("BIJUX_TELEMETRY_JSONL").map_or_else(
        |_| run_artifacts_dir.join("telemetry").join("events.jsonl"),
        PathBuf::from,
    );
    let canonical_params = parameters_json_canonicalization(&plan.params);
    let sample_id = canonical_params
        .get("sample_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_string();
    let params_hash = params_hash(&canonical_params)?;
    let adapter_bank = adapter_bank_from_params(&canonical_params);
    let banks_json = banks_from_params(&canonical_params);
    let bank_assets = materialize_bank_assets(&run_artifacts_dir, banks_json.as_ref())?;
    let input_paths: Vec<PathBuf> = plan
        .io
        .inputs
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect();
    let input_hashes: Vec<String> = input_paths
        .iter()
        .map(|path| hash_file_sha256(path))
        .collect::<Result<Vec<_>>>()?;
    let input_hash = hash_inputs(&input_paths)?;
    let metric_context =
        metric_context_from_params(plan, runner, &input_hash, &params_hash, &canonical_params);
    let plan_artifacts = write_plan_artifacts(
        &run_artifacts_dir,
        &plan.stage_id.0,
        stage_version_i32(plan.stage_version),
        &plan.tool_id.0,
        &plan.tool_version,
        plan.image.digest.clone(),
        &runner.to_string(),
        &std::env::var("BIJUX_PLATFORM").unwrap_or_else(|_| "unknown".to_string()),
        &plan.resources,
        &plan
            .io
            .inputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>(),
        &plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>(),
        &canonical_params,
        &plan.effective_params,
        adapter_bank.as_ref(),
        banks_json.as_ref(),
        bank_assets.as_ref(),
    )?;
    let image = resolved_image_for_plan(&plan.image, runner);
    let image_digest = plan
        .image
        .digest
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let emit_event = |event: &bijux_core::TelemetryEventV1| -> Result<()> {
        write_telemetry_event(&telemetry_path, event)?;
        write_stage_event_jsonl(&run_artifacts_dir, event)?;
        Ok(())
    };
    let emit_artifact = |name: &str, path: &Path| -> Result<()> {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "artifact_written".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "artifact": name,
                "path": path.display().to_string(),
            }),
        })
    };
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "stage_start".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "tool_version": plan.tool_version.clone(),
        }),
    })?;
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "tool_start".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "tool_version": plan.tool_version.clone(),
        }),
    })?;
    let started_at = Utc::now();
    let start = Instant::now();
    let mut telemetry_exit_code: Option<i32> = None;
    let mut telemetry_output_hashes: Vec<String> = Vec::new();
    let mut telemetry_error: Option<String> = None;
    if let Some(observer) = observer.as_mut() {
        let start_result =
            observer_result_from_plan(plan, Vec::new(), -1, String::new(), String::new());
        observer.on_stage_start(&start_result)?;
    }
    info!(
        run_id = %run_id,
        sample_id = %sample_id,
        stage = %plan.stage_id.0,
        tool = %plan.tool_id.0,
        tool_version = %plan.tool_version,
        image_digest = %plan.image.digest.clone().unwrap_or_else(|| "unknown".to_string()),
        params_hash = %params_hash,
        input_hash = %input_hash,
        "stage execution starting"
    );
    let result: Result<StageResultV1> = (|| {
        let run_result = run_stage_execution(
            plan,
            &image,
            runner,
            r1_dir,
            r1,
            r2,
            &container_name,
            &canonical_params,
        )?;
        telemetry_exit_code = Some(run_result.envelope.exit_code);
        let runtime_s = start.elapsed().as_secs_f64();
        let memory_mb = execution_memory_mb(&container_name)?;
        cleanup_execution(&container_name)?;
        let outputs = run_result.outputs_override.unwrap_or_else(|| {
            plan.io
                .outputs
                .iter()
                .map(|artifact| artifact.path.clone())
                .collect()
        });
        let post = process_execution_result(
            plan,
            &run_id,
            &sample_id,
            &run_artifacts_dir,
            &trace_id,
            &span_id,
            started_at,
            &canonical_params,
            &params_hash,
            adapter_bank.as_ref(),
            banks_json.as_ref(),
            bank_assets.as_ref(),
            &input_paths,
            &input_hashes,
            &input_hash,
            &metric_context,
            &plan_artifacts,
            &image_digest,
            &runner.to_string(),
            run_result.envelope,
            outputs,
            runtime_s,
            memory_mb,
            &emit_event,
            &emit_artifact,
        )?;
        telemetry_output_hashes.clone_from(&post.output_hashes);
        let stage_result = post.stage_result;
        if let Some(observer) = observer.as_mut() {
            let observer_result = observer_result_from_plan(
                plan,
                stage_result.outputs.clone(),
                stage_result.exit_code,
                stage_result.stdout.clone(),
                stage_result.stderr.clone(),
            );
            observer.on_stage_end(&observer_result)?;
        }
        Ok(stage_result)
    })();
    let runtime_s = start.elapsed().as_secs_f64();
    let duration_ms = {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            (runtime_s * 1000.0).max(0.0) as u64
        }
    };
    if let Err(err) = &result {
        let _ = cleanup_execution(&container_name);
        telemetry_error = Some(err.to_string());
    }
    let status = match telemetry_exit_code {
        Some(0) if result.is_ok() => "ok",
        _ => "error",
    };
    let exit_code = telemetry_exit_code.unwrap_or(-1);
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "tool_end".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: Some(duration_ms),
        status: status.to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "exit_code": exit_code,
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "output_hashes": &telemetry_output_hashes,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "error": telemetry_error.clone(),
        }),
    })?;
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "stage_end".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: Some(duration_ms),
        status: status.to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "exit_code": exit_code,
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "output_hashes": &telemetry_output_hashes,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "error": telemetry_error.clone(),
        }),
    })?;
    if let Some(error) = telemetry_error.as_ref() {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "error".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "error".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "message": error,
                "exit_code": exit_code,
            }),
        })?;
    }
    result
}
