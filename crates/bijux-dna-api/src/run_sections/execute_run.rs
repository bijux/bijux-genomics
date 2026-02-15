/// # Errors
/// Returns an error if execution fails.
pub fn execute_run(request: &ExecuteRunRequest) -> Result<ExecuteRunResult> {
    let runner_contract = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Docker => RunnerContractKind::Docker,
        bijux_dna_environment::api::RuntimeKind::Apptainer => RunnerContractKind::Apptainer,
        other @ bijux_dna_environment::api::RuntimeKind::Singularity => {
            return Err(anyhow!(
                "runner {other} not supported for execute_run stage coverage"
            ));
        }
    };
    ensure_stage_supported_by_runner(runner_contract, request.plan.stage_id.as_str())?;
    if hpc_context_enabled() {
        enforce_hpc_results_layout(&request.plan.out_dir)?;
    }
    let started_at = Instant::now();
    let run_id = format!("{}__{}", request.plan.stage_id, request.plan.tool_id);
    let run_artifacts_dir = request.plan.out_dir.join("run_artifacts");
    bijux_dna_infra::ensure_dir(&run_artifacts_dir)?;
    maybe_emit_reference_manifest(request, &run_artifacts_dir)?;
    let regime_stamp = resolve_and_write_regime_stamp(request, &run_artifacts_dir)?;
    let telemetry_path = run_artifacts_dir.join("telemetry.jsonl");
    let trace_id = format!("trace-{}", request.plan.stage_id);
    let span_id = format!("span-{}", request.plan.tool_id);
    let stage_span = info_span!(
        "stage_execute",
        stage_id = %request.plan.stage_id,
        tool_id = %request.plan.tool_id
    );
    let _entered = stage_span.enter();
    let stage_start = bijux_dna_runtime::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: request.plan.stage_id.to_string(),
        tool_id: request.plan.tool_id.to_string(),
        event_name: bijux_dna_runtime::TelemetryEventName::StageStart,
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: "running".to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: std::collections::BTreeMap::new(),
        failure_code: None,
    };
    if let Err(err) =
        bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &stage_start)
    {
        warn!("failed to write stage_start telemetry: {err}");
    }
    let manifest_hash = bijux_dna_core::contract::canonical::to_canonical_json_bytes(
        &bijux_dna_stage_contract::StagePlanJsonV1::from_plan(&request.plan),
    )
    .map(|bytes| {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    })?;
    let params_hash = bijux_dna_core::prelude::hashing::params_hash(&request.plan.params)?;
    let idempotent = request
        .plan
        .reason
        .details
        .get("idempotent")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let resume_meta_path = request
        .plan
        .out_dir
        .join("run_artifacts")
        .join("stage_resume.json");
    let output_checksums = || -> std::collections::BTreeMap<String, String> {
        let mut checksums = std::collections::BTreeMap::new();
        for artifact in &request.plan.io.outputs {
            let path = request.plan.out_dir.join(&artifact.path);
            if path.exists() {
                if let Ok(sum) = bijux_dna_runtime::recording::hash_file_sha256(&path) {
                    checksums.insert(artifact.name.to_string(), sum);
                }
            }
        }
        checksums
    };
    if idempotent {
        let outputs_exist = request.plan.io.outputs.iter().all(|artifact| {
            let path = request.plan.out_dir.join(&artifact.path);
            path.exists()
        });
        if outputs_exist && resume_meta_path.exists() {
            let meta_raw = std::fs::read_to_string(&resume_meta_path)
                .with_context(|| format!("read {}", resume_meta_path.display()))?;
            let meta: serde_json::Value = serde_json::from_str(&meta_raw)
                .with_context(|| format!("parse {}", resume_meta_path.display()))?;
            let same_manifest = meta
                .get("manifest_hash")
                .and_then(serde_json::Value::as_str)
                == Some(manifest_hash.as_str());
            let same_checksums = meta
                .get("output_checksums")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|existing| {
                    let current = output_checksums();
                    existing.iter().all(|(key, value)| {
                        value.as_str().is_some_and(|stored| {
                            current.get(key).is_some_and(|actual| actual == stored)
                        })
                    })
                });
            if same_manifest && same_checksums {
                let stage_end = bijux_dna_runtime::TelemetryEventV1 {
                    schema_version: "bijux.telemetry.v1".to_string(),
                    run_id: run_id.clone(),
                    stage_id: request.plan.stage_id.to_string(),
                    tool_id: request.plan.tool_id.to_string(),
                    event_name: bijux_dna_runtime::TelemetryEventName::StageEnd,
                    timestamp: chrono::Utc::now(),
                    duration_ms: Some(millis_u64(started_at.elapsed())),
                    status: "skipped".to_string(),
                    trace_id,
                    span_id,
                    attrs: std::collections::BTreeMap::from([(
                        "resume_reason".to_string(),
                        bijux_dna_runtime::AttrValue::Str("idempotent_manifest_match".to_string()),
                    )]),
                    failure_code: None,
                };
                if let Err(err) =
                    bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &stage_end)
                {
                    warn!("failed to write stage_end telemetry: {err}");
                }
                return Ok(ExecuteRunResult);
            }
        }
    }
    let tool_event = bijux_dna_runtime::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: request.plan.stage_id.to_string(),
        tool_id: request.plan.tool_id.to_string(),
        event_name: bijux_dna_runtime::TelemetryEventName::ToolInvocation,
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: "running".to_string(),
        trace_id: format!("trace-{}", request.plan.stage_id),
        span_id: format!("span-{}", request.plan.tool_id),
        attrs: bijux_dna_runtime::redacted_attrs(&std::collections::BTreeMap::from([
            (
                "runner".to_string(),
                bijux_dna_runtime::AttrValue::Str(request.runner.to_string()),
            ),
            (
                "stdout_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request
                        .plan
                        .out_dir
                        .join("logs")
                        .join("stdout.log")
                        .display()
                        .to_string(),
                ),
            ),
            (
                "stderr_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request
                        .plan
                        .out_dir
                        .join("logs")
                        .join("stderr.log")
                        .display()
                        .to_string(),
                ),
            ),
        ])),
        failure_code: None,
    };
    if let Err(err) =
        bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &tool_event)
    {
        warn!("failed to write tool_invocation telemetry: {err}");
    }
    let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&request.plan);
    let tmp_root = std::env::var("TMPDIR")
        .map_or_else(|_| run_artifacts_dir.join("tmp"), PathBuf::from)
        .join(&run_id);
    let input_root = request
        .plan
        .io
        .inputs
        .first()
        .and_then(|artifact| {
            request
                .plan
                .out_dir
                .join(&artifact.path)
                .parent()
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| request.plan.out_dir.clone());
    let network_policy = if request
        .plan
        .reason
        .details
        .get("network")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case("forbid"))
    {
        crate::execution_kernel::NetworkPolicy::Forbid
    } else {
        crate::execution_kernel::NetworkPolicy::Allow
    };
    let context = crate::execution_kernel::ToolContext {
        run_id: run_id.clone(),
        stage_id: request.plan.stage_id.to_string(),
        tool_id: request.plan.tool_id.to_string(),
        sample_id: request
            .plan
            .reason
            .details
            .get("sample_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        stage_root: run_artifacts_dir.clone(),
        input_root,
        output_root: request.plan.out_dir.clone(),
        tmp_root: tmp_root.clone(),
        threads: request.plan.resources.threads.max(1),
        memory_hint_mb: Some(u64::from(request.plan.resources.mem_gb).saturating_mul(1024)),
        compression_threads: Some(1),
        seed: request
            .plan
            .reason
            .details
            .get("seed")
            .and_then(serde_json::Value::as_u64),
        network_policy,
    };
    let invocation_request = crate::execution_kernel::ToolInvocationRequest {
        step: step.clone(),
        runner: request.runner,
        context,
        timeout: None,
        mode: crate::execution_kernel::ToolExecMode::Execute,
    };
    let invocation_result = match crate::execution_kernel::ToolExec::invoke(&invocation_request) {
        Ok(result) => result,
        Err(err) => {
            let fail_code = if err.to_string().contains("path contract violated")
                || err.to_string().contains("network policy violation")
            {
                bijux_dna_runtime::FailureCode::InvariantViolation
            } else {
                bijux_dna_runtime::FailureCode::ToolFailed
            };
            let fail_event = bijux_dna_runtime::TelemetryEventV1 {
                schema_version: "bijux.telemetry.v1".to_string(),
                run_id: run_id.clone(),
                stage_id: request.plan.stage_id.to_string(),
                tool_id: request.plan.tool_id.to_string(),
                event_name: bijux_dna_runtime::TelemetryEventName::RunFailed,
                timestamp: chrono::Utc::now(),
                duration_ms: Some(millis_u64(started_at.elapsed())),
                status: "error".to_string(),
                trace_id: format!("trace-{}", request.plan.stage_id),
                span_id: format!("span-{}", request.plan.tool_id),
                attrs: std::collections::BTreeMap::from([(
                    "error".to_string(),
                    bijux_dna_runtime::AttrValue::Str(err.to_string()),
                )]),
                failure_code: Some(fail_code),
            };
            let _ =
                bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &fail_event);
            return Err(err);
        }
    };
    if invocation_result.stage_result.exit_code != 0 {
        let err = anyhow!(
            "stage {} failed with exit code {}",
            request.plan.stage_id,
            invocation_result.stage_result.exit_code
        );
        let fail_event = bijux_dna_runtime::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: request.plan.stage_id.to_string(),
            tool_id: request.plan.tool_id.to_string(),
            event_name: bijux_dna_runtime::TelemetryEventName::RunFailed,
            timestamp: chrono::Utc::now(),
            duration_ms: Some(millis_u64(started_at.elapsed())),
            status: "error".to_string(),
            trace_id: format!("trace-{}", request.plan.stage_id),
            span_id: format!("span-{}", request.plan.tool_id),
            attrs: std::collections::BTreeMap::from([(
                "error".to_string(),
                bijux_dna_runtime::AttrValue::Str(err.to_string()),
            )]),
            failure_code: Some(bijux_dna_runtime::FailureCode::ToolFailed),
        };
        let _ = bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &fail_event);
        return Err(err);
    }
    if std::env::var("BIJUX_RUNTIME_PARITY_CHECK")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
    {
        let secondary_runner = match request.runner {
            bijux_dna_environment::api::RuntimeKind::Docker => {
                bijux_dna_environment::api::RuntimeKind::Apptainer
            }
            bijux_dna_environment::api::RuntimeKind::Apptainer => {
                bijux_dna_environment::api::RuntimeKind::Docker
            }
            bijux_dna_environment::api::RuntimeKind::Singularity => {
                bijux_dna_environment::api::RuntimeKind::Singularity
            }
        };
        if secondary_runner != request.runner {
            let parity = crate::cross_runtime::check_invocation_parity(
                &invocation_request,
                secondary_runner,
            )?;
            bijux_dna_infra::atomic_write_json(
                &run_artifacts_dir.join("runtime_parity.json"),
                &parity,
            )?;
        }
    }
    let params_hash_path = run_artifacts_dir.join("stage_params_hash.json");
    let params_hash_payload = serde_json::json!({
        "schema_version": "bijux.stage_params_hash.v1",
        "stage_id": request.plan.stage_id,
        "params_hash": params_hash,
        "manifest_hash": manifest_hash,
        "stage_semver": request.plan.stage_version.0,
    });
    bijux_dna_infra::atomic_write_json(&params_hash_path, &params_hash_payload)?;
    let explain_path = run_artifacts_dir.join("explain.json");
    let explain_payload = serde_json::json!({
        "schema_version": "bijux.stage_explain.v1",
        "stage_id": request.plan.stage_id,
        "tool_id": request.plan.tool_id,
        "summary": request.plan.reason.summary,
        "decision_kind": format!("{:?}", request.plan.reason.kind),
        "decision_details": request.plan.reason.details,
        "resources": {
            "threads": request.plan.resources.threads,
            "mem_gb": request.plan.resources.mem_gb,
            "tmp_gb": request.plan.resources.tmp_gb,
        },
        "io": {
            "inputs": request.plan.io.inputs,
            "outputs": request.plan.io.outputs,
        },
        "coverage_regime": regime_stamp,
    });
    bijux_dna_infra::atomic_write_json(&explain_path, &explain_payload)?;
    let resume_payload = serde_json::json!({
        "schema_version": "bijux.stage_resume.v1",
        "manifest_hash": manifest_hash,
        "params_hash": params_hash,
        "stage_semver": request.plan.stage_version.0,
        "idempotent": idempotent,
        "output_checksums": output_checksums(),
    });
    bijux_dna_infra::atomic_write_json(&resume_meta_path, &resume_payload)?;
    for artifact in &request.plan.io.outputs {
        let event = bijux_dna_runtime::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: request.plan.stage_id.to_string(),
            tool_id: request.plan.tool_id.to_string(),
            event_name: bijux_dna_runtime::TelemetryEventName::ArtifactWritten,
            timestamp: chrono::Utc::now(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: format!("trace-{}", request.plan.stage_id),
            span_id: format!("span-{}", request.plan.tool_id),
            attrs: std::collections::BTreeMap::from([
                (
                    "artifact_id".to_string(),
                    bijux_dna_runtime::AttrValue::Str(artifact.name.to_string()),
                ),
                (
                    "artifact_path".to_string(),
                    bijux_dna_runtime::AttrValue::Str(artifact.path.display().to_string()),
                ),
            ]),
            failure_code: None,
        };
        let _ = bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &event);
    }
    let metrics_event = bijux_dna_runtime::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: request.plan.stage_id.to_string(),
        tool_id: request.plan.tool_id.to_string(),
        event_name: bijux_dna_runtime::TelemetryEventName::MetricsEmitted,
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: format!("trace-{}", request.plan.stage_id),
        span_id: format!("span-{}", request.plan.tool_id),
        attrs: std::collections::BTreeMap::from([(
            "metrics_path".to_string(),
            bijux_dna_runtime::AttrValue::Str(
                request
                    .plan
                    .out_dir
                    .join("metrics.json")
                    .display()
                    .to_string(),
            ),
        )]),
        failure_code: None,
    };
    let _ = bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &metrics_event);
    let invariant_event = bijux_dna_runtime::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: request.plan.stage_id.to_string(),
        tool_id: request.plan.tool_id.to_string(),
        event_name: bijux_dna_runtime::TelemetryEventName::InvariantResult,
        timestamp: chrono::Utc::now(),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: format!("trace-{}", request.plan.stage_id),
        span_id: format!("span-{}", request.plan.tool_id),
        attrs: std::collections::BTreeMap::from([(
            "manifest_hash".to_string(),
            bijux_dna_runtime::AttrValue::Str(manifest_hash.clone()),
        )]),
        failure_code: None,
    };
    let _ = bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &invariant_event);
    let stage_end = bijux_dna_runtime::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: request.plan.stage_id.to_string(),
        tool_id: request.plan.tool_id.to_string(),
        event_name: bijux_dna_runtime::TelemetryEventName::StageEnd,
        timestamp: chrono::Utc::now(),
        duration_ms: Some(millis_u64(started_at.elapsed())),
        status: "ok".to_string(),
        trace_id: format!("trace-{}", request.plan.stage_id),
        span_id: format!("span-{}", request.plan.tool_id),
        attrs: std::collections::BTreeMap::from([
            (
                "bytes_written".to_string(),
                bijux_dna_runtime::AttrValue::Int(
                    request
                        .plan
                        .io
                        .outputs
                        .iter()
                        .filter_map(|artifact| {
                            let path = request.plan.out_dir.join(&artifact.path);
                            std::fs::metadata(path).ok().map(|m| file_len_i64(m.len()))
                        })
                        .sum(),
                ),
            ),
            (
                "stdout_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request
                        .plan
                        .out_dir
                        .join("logs")
                        .join("stdout.log")
                        .display()
                        .to_string(),
                ),
            ),
            (
                "stderr_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request
                        .plan
                        .out_dir
                        .join("logs")
                        .join("stderr.log")
                        .display()
                        .to_string(),
                ),
            ),
        ]),
        failure_code: None,
    };
    let _ = bijux_dna_runtime::recording::write_telemetry_event(&telemetry_path, &stage_end);
    let compact_summary = serde_json::json!({
        "schema_version": "bijux.telemetry_run_summary.v1",
        "run_id": run_id,
        "stage_id": request.plan.stage_id.to_string(),
        "tool_id": request.plan.tool_id.to_string(),
        "status": "ok",
        "runtime_ms": millis_u64(started_at.elapsed()),
        "telemetry_path": telemetry_path.display().to_string(),
    });
    bijux_dna_infra::atomic_write_json(
        &run_artifacts_dir.join("run_summary.json"),
        &compact_summary,
    )?;
    maybe_write_site_lock(&request.plan.out_dir)?;
    let _ = bijux_dna_infra::remove_dir_all(&tmp_root);
    Ok(ExecuteRunResult)
}

fn maybe_emit_reference_manifest(
    request: &ExecuteRunRequest,
    run_artifacts_dir: &std::path::Path,
) -> Result<()> {
    let reference_inputs = request
        .plan
        .io
        .inputs
        .iter()
        .filter(|artifact| {
            let name = artifact.name.to_string().to_ascii_lowercase();
            name.contains("reference") || name.contains("fasta") || name.contains("ref")
        })
        .collect::<Vec<_>>();
    if reference_inputs.is_empty() {
        return Ok(());
    }

    let species = request
        .plan
        .params
        .get("species_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let build = request
        .plan
        .params
        .get("build_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let usecase = request
        .plan
        .params
        .get("usecase")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    let mut inputs = Vec::new();
    for artifact in reference_inputs {
        let resolved = request.plan.out_dir.join(&artifact.path);
        let sha256 = if resolved.exists() {
            Some(bijux_dna_infra::hash_file_sha256(&resolved)?)
        } else {
            None
        };
        inputs.push(serde_json::json!({
            "name": artifact.name,
            "path": resolved,
            "sha256": sha256,
        }));
    }

    let authority =
        if let (Some(species_id), Some(build_id)) = (species.as_deref(), build.as_deref()) {
            let bundle = bijux_dna_db_ref::resolve_reference_bundle(species_id, build_id)?;
            let bank = bijux_dna_db_ref::resolve_reference_bank(species_id, build_id)?;
            let sex_rule = bijux_dna_db_ref::resolve_sex_chromosome_rule(species_id, build_id).ok();
            let organellar = bijux_dna_db_ref::resolve_organellar_policy(species_id, build_id).ok();
            let default_set = usecase.as_deref().and_then(|kind| {
                bijux_dna_db_ref::resolve_default_reference_set(species_id, kind).ok()
            });
            Some(serde_json::json!({
                "species_id": species_id,
                "build_id": build_id,
                "bundle": bundle,
                "bank": bank,
                "sex_chromosome_rule": sex_rule,
                "organellar_policy": organellar,
                "default_reference_set": default_set,
            }))
        } else {
            None
        };

    let payload = serde_json::json!({
        "schema_version": "bijux.reference_manifest.v1",
        "stage_id": request.plan.stage_id.to_string(),
        "tool_id": request.plan.tool_id.to_string(),
        "reference_inputs": inputs,
        "authority": authority,
    });
    bijux_dna_infra::atomic_write_json(
        &run_artifacts_dir.join("reference_manifest.json"),
        &payload,
    )?;
    Ok(())
}

fn resolve_and_write_regime_stamp(
    request: &ExecuteRunRequest,
    run_artifacts_dir: &std::path::Path,
) -> Result<serde_json::Value> {
    let stamp = resolve_regime_stamp(request)?;
    bijux_dna_infra::atomic_write_json(&run_artifacts_dir.join("regime_stamp.json"), &stamp)?;
    Ok(stamp)
}

fn resolve_regime_stamp(request: &ExecuteRunRequest) -> Result<serde_json::Value> {
    let requested = request
        .plan
        .params
        .get("coverage_regime")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let profile = request
        .plan
        .params
        .get("regime_profile")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("default");
    let thresholds = load_coverage_thresholds(profile)?;

    let observed_bam_mean_depth = request
        .plan
        .params
        .get("mean_depth_x")
        .and_then(serde_json::Value::as_f64)
        .or_else(|| {
            let path = request
                .plan
                .out_dir
                .join("bam")
                .join("coverage")
                .join("coverage.regime.json");
            if !path.exists() {
                return None;
            }
            std::fs::read_to_string(path).ok().and_then(|raw| {
                serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|json| json.get("mean_depth").and_then(serde_json::Value::as_f64))
            })
        });

    let observed_fastq_estimated_depth = classify_fastq_only_estimated_depth(request);
    let observed_mean_depth = observed_bam_mean_depth.or(observed_fastq_estimated_depth);
    let observed_variant_density = classify_vcf_variant_density(request);

    let (selected, trigger) = if let Some(mean_depth) = observed_mean_depth {
        classify_depth_to_regime(mean_depth, thresholds)
    } else if let Some(regime) = requested.as_deref() {
        (
            regime,
            "requested coverage_regime used because observed mean depth is unavailable".to_string(),
        )
    } else if stage_requires_regime(request.plan.stage_id.as_str()) {
        return Err(anyhow!(
            "coverage regime required for stage {} but cannot be resolved: provide coverage_regime or mean_depth_x/FASTQ expected genome size inputs",
            request.plan.stage_id
        ));
    } else {
        (
            "unknown",
            "regime not required for stage and no observed/requested signal available".to_string(),
        )
    };

    let (impute_backend_default, chunk_size_bp_default) = match selected {
        "gl" => ("glimpse", 2_000_000_u64),
        "pseudohaploid" => ("beagle", 3_000_000_u64),
        "diploid" => ("minimac4", 5_000_000_u64),
        _ => ("unknown", 0_u64),
    };

    Ok(serde_json::json!({
        "schema_version": "bijux.coverage_regime_stamp.v1",
        "stage_id": request.plan.stage_id.to_string(),
        "selected_regime": selected,
        "requested_regime": requested,
        "regime_profile": profile,
        "trigger": trigger,
        "thresholds_used": {
            "gl_max_depth": thresholds.gl_max_depth,
            "pseudohaploid_max_depth": thresholds.pseudohaploid_max_depth,
            "diploid_min_depth": thresholds.diploid_min_depth,
        },
        "observed_coverage_stats": {
            "mean_depth_x": observed_mean_depth,
            "bam_mean_depth_x": observed_bam_mean_depth,
            "fastq_estimated_depth_x": observed_fastq_estimated_depth,
            "variant_density_per_mb": observed_variant_density,
        },
        "routing": {
            "call_stage": match selected {
                "gl" => "vcf.call_gl",
                "pseudohaploid" => "vcf.call_pseudohaploid",
                "diploid" => "vcf.call_diploid",
                _ => "unknown",
            },
            "imputation_backend_default": impute_backend_default,
            "chunk_size_bp_default": chunk_size_bp_default,
        }
    }))
}

fn classify_depth_to_regime(
    mean_depth: f64,
    thresholds: CoverageThresholds,
) -> (&'static str, String) {
    if mean_depth <= thresholds.gl_max_depth {
        (
            "gl",
            format!(
                "mean_depth_x <= gl_max_depth ({mean_depth:.4} <= {})",
                thresholds.gl_max_depth
            ),
        )
    } else if mean_depth <= thresholds.pseudohaploid_max_depth {
        (
            "pseudohaploid",
            format!(
                "gl_max_depth < mean_depth_x <= pseudohaploid_max_depth ({} < {mean_depth:.4} <= {})",
                thresholds.gl_max_depth, thresholds.pseudohaploid_max_depth
            ),
        )
    } else if mean_depth >= thresholds.diploid_min_depth {
        (
            "diploid",
            format!(
                "mean_depth_x >= diploid_min_depth ({mean_depth:.4} >= {})",
                thresholds.diploid_min_depth
            ),
        )
    } else {
        (
            "pseudohaploid",
            "fallback band between pseudohaploid_max_depth and diploid_min_depth".to_string(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct CoverageThresholds {
    gl_max_depth: f64,
    pseudohaploid_max_depth: f64,
    diploid_min_depth: f64,
}

fn load_coverage_thresholds(profile: &str) -> Result<CoverageThresholds> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf);
    let raw = std::fs::read_to_string(root.join("configs/runtime/coverage_regimes.toml"))?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let decision = parsed
        .get("decision")
        .and_then(|v| v.get("coverage_regime"))
        .ok_or_else(|| anyhow!("missing decision.coverage_regime in coverage_regimes.toml"))?;
    let base = decision
        .get("thresholds")
        .ok_or_else(|| anyhow!("missing decision.coverage_regime.thresholds"))?;
    let profile_thresholds = decision
        .get("profiles")
        .and_then(|v| v.get(profile))
        .unwrap_or(base);
    let read_f = |obj: &toml::Value, key: &str| -> Result<f64> {
        obj.get(key)
            .and_then(toml::Value::as_float)
            .or_else(|| {
                obj.get(key)
                    .and_then(toml::Value::as_integer)
                    .map(|v| v as f64)
            })
            .ok_or_else(|| anyhow!("missing or invalid threshold key `{key}`"))
    };
    Ok(CoverageThresholds {
        gl_max_depth: read_f(profile_thresholds, "gl_max_depth")?,
        pseudohaploid_max_depth: read_f(profile_thresholds, "pseudohaploid_max_depth")?,
        diploid_min_depth: read_f(profile_thresholds, "diploid_min_depth")?,
    })
}

fn stage_requires_regime(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "vcf.call"
            | "vcf.call_gl"
            | "vcf.call_diploid"
            | "vcf.call_pseudohaploid"
            | "vcf.impute"
            | "vcf.phasing"
    )
}

fn classify_fastq_only_estimated_depth(request: &ExecuteRunRequest) -> Option<f64> {
    let expected_genome_size_bp = request
        .plan
        .params
        .get("expected_genome_size_bp")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| {
            std::env::var("BIJUX_EXPECTED_GENOME_SIZE_BP")
                .ok()?
                .parse::<u64>()
                .ok()
        })?;
    if expected_genome_size_bp == 0 {
        return None;
    }
    let mut read_count: u64 = 0;
    let mut mean_len_sum: f64 = 0.0;
    let mut files: u64 = 0;
    for artifact in &request.plan.io.inputs {
        let p = request.plan.out_dir.join(&artifact.path);
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or_default();
        if !(ext.eq_ignore_ascii_case("fq")
            || ext.eq_ignore_ascii_case("fastq")
            || ext.eq_ignore_ascii_case("gz"))
        {
            continue;
        }
        let inv = request.plan.out_dir.join("fastq_invariants.json");
        if inv.exists() {
            if let Ok(raw) = std::fs::read_to_string(&inv) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                    let r1_count = json
                        .pointer("/r1/read_count")
                        .and_then(serde_json::Value::as_u64);
                    let r1_len = json
                        .pointer("/r1/read_length_mean")
                        .and_then(serde_json::Value::as_f64);
                    if let (Some(c), Some(l)) = (r1_count, r1_len) {
                        read_count = read_count.saturating_add(c);
                        mean_len_sum += l;
                        files = files.saturating_add(1);
                    }
                }
            }
        }
    }
    if read_count == 0 || files == 0 {
        return None;
    }
    let avg_len = mean_len_sum / files as f64;
    Some((read_count as f64 * avg_len) / expected_genome_size_bp as f64)
}

fn classify_vcf_variant_density(request: &ExecuteRunRequest) -> Option<f64> {
    let mut variants: u64 = 0;
    let mut span_bp: u64 = 0;
    for artifact in &request.plan.io.inputs {
        let p = request.plan.out_dir.join(&artifact.path);
        let name = p.file_name().and_then(|x| x.to_str()).unwrap_or_default();
        if !(name.ends_with(".vcf") || name.ends_with(".vcf.gz")) {
            continue;
        }
        let raw = if name.ends_with(".gz") {
            std::process::Command::new("gzip")
                .args(["-cd", p.to_string_lossy().as_ref()])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        } else {
            std::fs::read_to_string(&p).ok()
        }?;
        let mut contig_max = std::collections::BTreeMap::<String, u64>::new();
        for line in raw.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let chr = cols.next()?.to_string();
            let pos = cols.next()?.parse::<u64>().ok()?;
            variants = variants.saturating_add(1);
            let entry = contig_max.entry(chr).or_insert(0);
            *entry = (*entry).max(pos);
        }
        span_bp = span_bp.saturating_add(contig_max.values().sum::<u64>());
    }
    if variants == 0 || span_bp == 0 {
        return None;
    }
    Some(variants as f64 / (span_bp as f64 / 1_000_000_f64))
}

#[cfg(test)]
mod coverage_regime_tests {
    use super::{classify_depth_to_regime, CoverageThresholds};

    #[test]
    fn same_input_depth_yields_same_regime_deterministically() {
        let t = CoverageThresholds {
            gl_max_depth: 1.5,
            pseudohaploid_max_depth: 6.0,
            diploid_min_depth: 8.0,
        };
        let (a, _) = classify_depth_to_regime(1.2, t);
        let (b, _) = classify_depth_to_regime(1.2, t);
        let (c, _) = classify_depth_to_regime(1.2, t);
        assert_eq!(a, "gl");
        assert_eq!(a, b);
        assert_eq!(b, c);
    }
}
