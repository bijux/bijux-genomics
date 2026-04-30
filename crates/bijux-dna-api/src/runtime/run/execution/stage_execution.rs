#![allow(clippy::too_many_lines)]

use super::super::execution_support::{
    maybe_emit_reference_manifest, resolve_and_write_regime_stamp,
};
use super::super::{
    anyhow, enforce_hpc_results_layout, ensure_stage_supported_by_runner, file_len_i64,
    hpc_context_enabled, info_span, maybe_write_site_lock, millis_u64, warn, Context,
    ExecuteRunRequest, ExecuteRunResult, Instant, Path, PathBuf, Result, RunnerContractKind,
};
use std::fmt::Write as _;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn common_input_root(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut iter = paths.iter();
    let first = iter.next()?.clone();
    let mut prefix = first;
    for path in iter {
        while !path.starts_with(&prefix) {
            if !prefix.pop() {
                return None;
            }
        }
    }
    Some(prefix)
}

/// # Errors
/// Returns an error if execution fails.
#[allow(clippy::too_many_lines)]
pub fn execute_run(request: &ExecuteRunRequest) -> Result<ExecuteRunResult> {
    let runner_contract = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Local => RunnerContractKind::Local,
        bijux_dna_environment::api::RuntimeKind::Docker => RunnerContractKind::Docker,
        bijux_dna_environment::api::RuntimeKind::Apptainer => RunnerContractKind::Apptainer,
        other @ bijux_dna_environment::api::RuntimeKind::Singularity => {
            return Err(anyhow!("runner {other} not supported for execute_run stage coverage"));
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
        sha256_hex(&hasher.finalize())
    })?;
    let params_hash = bijux_dna_core::prelude::hashing::params_hash(&request.plan.params)?;
    let idempotent = request
        .plan
        .reason
        .details
        .get("idempotent")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let resume_meta_path = request.plan.out_dir.join("run_artifacts").join("stage_resume.json");
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
            let same_manifest = meta.get("manifest_hash").and_then(serde_json::Value::as_str)
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
            ("runner".to_string(), bijux_dna_runtime::AttrValue::Str(request.runner.to_string())),
            (
                "stdout_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request.plan.out_dir.join("logs").join("stdout.log").display().to_string(),
                ),
            ),
            (
                "stderr_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request.plan.out_dir.join("logs").join("stderr.log").display().to_string(),
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
        .iter()
        .map(|artifact| request.plan.out_dir.join(&artifact.path))
        .map(|path| {
            if path.is_dir() {
                path
            } else {
                path.parent().map_or_else(|| path.clone(), Path::to_path_buf)
            }
        })
        .collect::<Vec<_>>();
    let input_root = common_input_root(&input_root).unwrap_or_else(|| request.plan.out_dir.clone());
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
        seed: request.plan.reason.details.get("seed").and_then(serde_json::Value::as_u64),
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
            bijux_dna_environment::api::RuntimeKind::Local => {
                bijux_dna_environment::api::RuntimeKind::Local
            }
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
                request.plan.out_dir.join("metrics.json").display().to_string(),
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
                    request.plan.out_dir.join("logs").join("stdout.log").display().to_string(),
                ),
            ),
            (
                "stderr_path".to_string(),
                bijux_dna_runtime::AttrValue::Str(
                    request.plan.out_dir.join("logs").join("stderr.log").display().to_string(),
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
