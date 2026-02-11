use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info_span, warn};

use crate::request_args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, PlanRunRequest, PlanRunResult,
    RenderReportRequest, RenderReportResult, RunRequest, RunResult, RunStatus,
};
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_core::contract::{Profile, RunSpec, ToolRegistry};
use bijux_dna_core::ids::RunId;
use bijux_dna_engine::Engine;
use bijux_dna_pipelines::registry::PipelineRegistry;
use bijux_dna_pipelines::{Domain, PipelineProfile};
use bijux_dna_runner::DockerRunner;
use bijux_dna_stage_contract::{build_run_execution_plan, RunExecutionPlan};
use cargo_metadata::MetadataCommand;

#[derive(Debug, Clone, Copy)]
/// Execution mode for pipeline runs.
///
/// Stability: v1 (stable).
pub enum RunMode {
    PlanOnly,
    Execute,
}

/// # Errors
/// Returns an error if the profile id is unknown or IO setup fails.
pub fn run_pipeline(request: RunRequest, _mode: RunMode) -> Result<RunResult> {
    let profile = bijux_dna_pipelines::registry::profile_by_id(request.domain, &request.profile_id)
        .map_err(|err| anyhow!("unknown pipeline profile {}: {err}", request.profile_id))?;
    bijux_dna_infra::ensure_dir(&request.run_dir)?;
    let ledger_path = request.run_dir.join("defaults_ledger.json");
    let defaults = profile.defaults_ledger();
    defaults.validate_strict()?;
    bijux_dna_infra::atomic_write_json(&ledger_path, &defaults)?;
    Ok(RunResult {
        run_dir: request.run_dir,
        profile_id: profile.id.to_string(),
    })
}

/// # Errors
/// Returns an error if the profile id is unknown.
pub fn select_pipeline(domain: Domain, profile_id: &str) -> Result<PipelineProfile> {
    bijux_dna_pipelines::registry::profile_by_id(domain, profile_id)
}

#[must_use]
pub fn select_pipelines(
    domain: Option<Domain>,
    include_experimental: bool,
) -> Vec<PipelineProfile> {
    let registry = PipelineRegistry::v1();
    if let Some(domain) = domain {
        registry
            .list_for_domain(domain, include_experimental)
            .into_iter()
            .cloned()
            .collect()
    } else {
        registry
            .list(include_experimental)
            .into_iter()
            .cloned()
            .collect()
    }
}

/// # Errors
/// Returns an error if planning fails for the requested run.
pub fn plan_run(request: PlanRunRequest, registry: &ToolRegistry) -> Result<PlanRunResult> {
    let plan = build_run_execution_plan(
        &request.run_spec,
        registry,
        &request.profile,
        request.run_id,
    )?;
    Ok(PlanRunResult { plan })
}

/// # Errors
/// Returns an error if planning fails for the requested run.
pub fn plan_only(request: PlanRunRequest, registry: &ToolRegistry) -> Result<PlanRunResult> {
    plan_run(request, registry)
}

fn millis_u64(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn file_len_i64(len: u64) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

fn hpc_context_enabled() -> bool {
    std::env::var("BIJUX_RUN_CONTEXT")
        .map(|v| v.eq_ignore_ascii_case("hpc"))
        .unwrap_or(false)
}

fn enforce_hpc_results_layout(out_dir: &Path) -> Result<()> {
    let comps = out_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(mut idx) = comps
        .iter()
        .position(|v| v == "results" || v == "bijux-dna-results")
    else {
        return Err(anyhow!("HPC run out_dir must be under results root"));
    };
    if comps.get(idx).is_some_and(|v| v == "bijux-dna-results")
        && comps.get(idx + 1).is_some_and(|v| v == "results")
    {
        idx += 1;
    }
    if comps.len() < idx + 7 {
        return Err(anyhow!(
            "HPC out_dir must match results/<corpus>/<pipeline>/<stage>/<tool>/<timestamp>/<run_id>"
        ));
    }
    let ts = &comps[idx + 5];
    let ts_ok = regex::Regex::new(r"^\d{8}T\d{6}Z$")
        .map(|re| re.is_match(ts))
        .unwrap_or(false);
    if !ts_ok {
        return Err(anyhow!("HPC out_dir timestamp must match YYYYMMDDTHHMMSSZ"));
    }
    Ok(())
}

fn maybe_write_site_lock(out_dir: &Path) -> Result<()> {
    if !hpc_context_enabled() {
        return Ok(());
    }
    let comps = out_dir.components().collect::<Vec<_>>();
    let results_idx = comps.iter().position(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "bijux-dna-results" || s == "results"
    });
    let Some(idx) = results_idx else {
        return Ok(());
    };
    let mut root = PathBuf::new();
    for comp in &comps[..=idx] {
        root.push(comp.as_os_str());
    }
    let lock_path = root.join("site_lock.json");
    let apptainer_version = std::process::Command::new("apptainer")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let kernel = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|raw| {
            raw.lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|v| v.trim().to_string())
        });
    let payload = serde_json::json!({
        "schema_version": "bijux.site_lock.v1",
        "site": std::env::var("BIJUX_HPC_SITE").unwrap_or_else(|_| "lunarc".to_string()),
        "apptainer_version": apptainer_version,
        "kernel": kernel,
        "cpu_model": cpu_model,
    });
    bijux_dna_infra::atomic_write_json(&lock_path, &payload)?;
    Ok(())
}

/// # Errors
/// Returns an error if execution fails.
#[allow(clippy::too_many_lines)]
pub fn execute_run(request: &ExecuteRunRequest) -> Result<ExecuteRunResult> {
    if hpc_context_enabled() {
        enforce_hpc_results_layout(&request.plan.out_dir)?;
    }
    let started_at = Instant::now();
    let run_id = format!("{}__{}", request.plan.stage_id, request.plan.tool_id);
    let run_artifacts_dir = request.plan.out_dir.join("run_artifacts");
    bijux_dna_infra::ensure_dir(&run_artifacts_dir)?;
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
            if same_manifest {
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
    let unique_tmp = if hpc_context_enabled() {
        let tmp_root =
            std::env::var("TMPDIR").map_or_else(|_| run_artifacts_dir.join("tmp"), PathBuf::from);
        let tmp = tmp_root.join(&run_id);
        bijux_dna_infra::ensure_dir(&tmp)?;
        std::env::set_var("TMPDIR", &tmp);
        Some(tmp)
    } else {
        None
    };
    if let Err(err) = bijux_dna_runner::execute::execute_step(&step, request.runner, None) {
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
    let params_hash_path = run_artifacts_dir.join("stage_params_hash.json");
    let params_hash_payload = serde_json::json!({
        "schema_version": "bijux.stage_params_hash.v1",
        "stage_id": request.plan.stage_id,
        "params_hash": params_hash,
        "manifest_hash": manifest_hash,
        "stage_semver": request.plan.stage_version.0,
    });
    bijux_dna_infra::atomic_write_json(&params_hash_path, &params_hash_payload)?;
    let resume_payload = serde_json::json!({
        "schema_version": "bijux.stage_resume.v1",
        "manifest_hash": manifest_hash,
        "params_hash": params_hash,
        "stage_semver": request.plan.stage_version.0,
        "idempotent": idempotent,
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
    if let Some(tmp) = unique_tmp {
        let _ = std::fs::remove_dir_all(tmp);
    }
    Ok(ExecuteRunResult)
}

/// # Errors
/// Returns an error if execution or report rendering fails.
pub fn execute_and_report(
    exec: &ExecuteRunRequest,
    report: &RenderReportRequest,
) -> Result<RenderReportResult> {
    execute_run(exec)?;
    render_report(report)
}

/// # Errors
/// Returns an error if report rendering fails.
pub fn render_report(request: &RenderReportRequest) -> Result<RenderReportResult> {
    let report_path = render_report_from_facts(&request.base_dir, &request.facts_path)?;
    Ok(RenderReportResult { report_path })
}

/// # Errors
/// Returns an error if run status inspection fails.
pub fn status(run_dir: &Path) -> Result<RunStatus> {
    let manifest_path = run_dir.join("run_manifest.json");
    let report_path =
        bijux_dna_runtime::recording::run_artifacts_dir_for_out(run_dir).join("report.html");
    let manifest = if manifest_path.exists() {
        Some(manifest_path.clone())
    } else {
        None
    };
    let report = if report_path.exists() {
        Some(report_path)
    } else {
        None
    };
    let has_failures = manifest
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|value| value.get("failures").cloned())
        .and_then(|value| value.as_array().cloned())
        .is_some_and(|failures| !failures.is_empty());
    Ok(RunStatus {
        run_dir: run_dir.to_path_buf(),
        manifest_path: manifest,
        report_path: report,
        has_failures,
    })
}

/// Replay or verify a run from a run manifest.
///
/// # Errors
/// Returns an error if manifest parsing, graph loading, execution, or verification fails.
pub fn replay_manifest(manifest_path: &Path, verify_only: bool) -> Result<()> {
    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|err| anyhow!("read run_manifest.json: {err}"))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&raw).map_err(|err| anyhow!("parse run_manifest.json: {err}"))?;
    let base_dir = manifest_path
        .parent()
        .ok_or_else(|| anyhow!("run_manifest.json missing parent"))?;
    let artifacts = manifest
        .get("output_artifacts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if verify_only {
        for entry in artifacts {
            let Some(path_value) = entry.get("path") else {
                continue;
            };
            let Some(path_str) = path_value.as_str() else {
                continue;
            };
            let path = base_dir.join(path_str);
            if !path.exists() {
                return Err(anyhow!("missing output artifact {}", path.display()));
            }
            if let Some(expected) = entry.get("sha256").and_then(|v| v.as_str()) {
                let actual = bijux_dna_infra::hash_file_sha256(&path)?;
                if actual != expected {
                    return Err(anyhow!(
                        "artifact hash mismatch for {} (expected {}, got {})",
                        path.display(),
                        expected,
                        actual
                    ));
                }
            }
        }
        return Ok(());
    }
    let graph_path =
        bijux_dna_runtime::recording::run_artifacts_dir_for_out(base_dir).join("graph.json");
    let graph_raw =
        std::fs::read_to_string(&graph_path).map_err(|err| anyhow!("read graph.json: {err}"))?;
    let graph: ExecutionGraph =
        serde_json::from_str(&graph_raw).map_err(|err| anyhow!("parse graph.json: {err}"))?;
    let runner = DockerRunner::new(None);
    let layout = run_layout_from_dir(base_dir);
    Engine::default().execute(&graph, &runner, &layout, None, None)?;
    Ok(())
}

/// # Errors
/// Returns an error if planning fails.
pub fn plan(request: PlanRequest) -> Result<PlanResponse> {
    let graph_hash = request.graph.hash()?;
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": "plan-only",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": bijux_dna_core::prelude::CacheKey::new("unknown", "unknown", "unknown", "unknown"),
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [
            {
                "kind": "graph",
                "schema": "bijux.execution_graph.v1",
                "path": "graph.json",
                "sha256": "unknown"
            },
            {
                "kind": "run_manifest",
                "schema": "bijux.run_manifest.v3",
                "path": "run_manifest.json",
                "sha256": "unknown"
            },
            {
                "kind": "run_summary",
                "schema": "bijux.run_summary.v1",
                "path": "run_summary.json",
                "sha256": "unknown"
            }
        ],
        "stages": [],
        "failures": [],
    });
    Ok(PlanResponse {
        graph: request.graph,
        graph_hash,
        manifest,
    })
}

/// # Errors
/// Returns an error if execution fails.
pub fn execute(request: &ExecuteRequest) -> Result<ExecuteResponse> {
    let (run_id, layout) = bijux_dna_runtime::run_layout::create_run_layout(&request.run_dir)?;
    let runner: Box<dyn bijux_dna_runtime::Runner> = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Docker => Box::new(DockerRunner::new(None)),
        other => {
            return Err(anyhow!("runner {other} not supported for execute"));
        }
    };
    Engine::default().execute(&request.graph, runner.as_ref(), &layout, None, None)?;
    let summary_path = layout.summary_dir.join("run_summary.json");
    write_run_summary_artifact(
        &summary_path,
        "execute",
        request.graph.pipeline_id().as_str(),
        &layout.manifest_path,
    )?;
    Ok(ExecuteResponse {
        run_id,
        manifest_path: layout.manifest_path,
        report_path: None,
    })
}

/// # Errors
/// Returns an error if dry-run output cannot be written.
pub fn dry_run(request: &DryRunRequest) -> Result<DryRunResponse> {
    let graph_hash = request.graph.hash()?;
    let graph_path = request.run_dir.join("graph.json");
    let graph_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&request.graph)?;
    bijux_dna_infra::atomic_write_bytes(&graph_path, graph_payload.as_slice())?;
    let mut manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": "dry-run",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": bijux_dna_core::prelude::CacheKey::new("unknown", "unknown", "unknown", "unknown"),
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [],
        "stages": [],
        "failures": [],
    });
    let manifest_path = request.run_dir.join("run_manifest.json");
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    let summary_path = request.run_dir.join("run_summary.json");
    write_run_summary_artifact(
        &summary_path,
        "dry-run",
        request.graph.pipeline_id().as_str(),
        &manifest_path,
    )?;
    let graph_sha = bijux_dna_infra::hash_file_sha256(&graph_path)?;
    let summary_sha = bijux_dna_infra::hash_file_sha256(&summary_path)?;
    manifest["output_artifacts"] = serde_json::json!([
        {
            "kind": "graph",
            "schema": "bijux.execution_graph.v1",
            "path": graph_path.display().to_string(),
            "sha256": graph_sha
        },
        {
            "kind": "run_summary",
            "schema": "bijux.run_summary.v1",
            "path": summary_path.display().to_string(),
            "sha256": summary_sha
        }
    ]);
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    let manifest_sha = bijux_dna_infra::hash_file_sha256(&manifest_path)?;
    if let Some(artifacts) = manifest["output_artifacts"].as_array_mut() {
        artifacts.push(serde_json::json!({
            "kind": "run_manifest",
            "schema": "bijux.run_manifest.v3",
            "path": manifest_path.display().to_string(),
            "sha256": manifest_sha
        }));
    } else {
        return Err(anyhow!("dry-run manifest output_artifacts is not an array"));
    }
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    bijux_dna_runtime::recording::write_profile_and_lock_manifests(&manifest_path)?;
    Ok(DryRunResponse {
        graph_path,
        manifest_path,
    })
}

fn write_run_summary_artifact(
    path: &Path,
    mode: &str,
    pipeline_id: &str,
    manifest_path: &Path,
) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "mode": mode,
        "pipeline_id": pipeline_id,
        "manifest_path": manifest_path.display().to_string(),
        "generated_at": Utc::now().to_rfc3339(),
    });
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    bijux_dna_infra::atomic_write_bytes(path, bytes.as_slice())?;
    Ok(())
}

/// # Errors
/// Returns an error if policy checks fail or cannot be executed.
pub fn policy_audit() -> Result<serde_json::Value> {
    let workspace = std::env::current_dir()?;
    let mut guardrails = serde_json::Map::new();
    for crate_name in ["bijux-dna-core", "bijux-dna-engine", "bijux-dna-api"] {
        let crate_root = workspace.join("crates").join(crate_name);
        let result = bijux_dna_policies::check(
            &crate_root,
            &bijux_dna_policies::GuardrailConfig::for_crate(crate_name),
        );
        let (status, error) = match result {
            Ok(()) => ("ok", None),
            Err(err) => ("fail", Some(err.to_string())),
        };
        guardrails.insert(
            crate_name.to_string(),
            serde_json::json!({
                "status": status,
                "error": error,
            }),
        );
    }
    Ok(serde_json::json!({
        "guardrails": guardrails,
    }))
}

/// # Errors
/// Returns an error if workspace dependency metadata cannot be loaded.
pub fn workspace_edges() -> Result<BTreeSet<(String, String)>> {
    let metadata = MetadataCommand::default()
        .exec()
        .context("exec cargo metadata")?;
    let workspace_members: HashSet<cargo_metadata::PackageId> =
        metadata.workspace_members.iter().cloned().collect();
    let mut id_to_name = HashMap::new();
    for pkg in &metadata.packages {
        id_to_name.insert(pkg.id.clone(), pkg.name.clone());
    }
    let mut edges = BTreeSet::new();
    if let Some(resolve) = metadata.resolve.as_ref() {
        for node in &resolve.nodes {
            let id = node.id.clone();
            if !workspace_members.contains(&id) {
                continue;
            }
            for dep in &node.deps {
                let dep_id = dep.pkg.clone();
                if !workspace_members.contains(&dep_id) {
                    continue;
                }
                let from = id_to_name
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| id.to_string());
                let to = id_to_name
                    .get(&dep_id)
                    .cloned()
                    .unwrap_or_else(|| dep_id.to_string());
                edges.insert((from, to));
            }
        }
    }
    Ok(edges)
}

/// # Errors
/// Returns an error if the workspace audit artifact cannot be written.
pub fn write_workspace_audit(out_dir: &Path, dot: &str) -> Result<PathBuf> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let dot_path = out_dir.join("graph.dot");
    bijux_dna_infra::write_bytes(&dot_path, dot.as_bytes())?;
    Ok(dot_path)
}

fn render_report_from_facts(base_dir: &Path, facts_path: &Path) -> Result<PathBuf> {
    let facts = bijux_dna_analyze::load::load_facts(facts_path)?;
    let report_path = bijux_dna_analyze::report::write_run_report_from_facts(base_dir, &facts)?;
    Ok(report_path)
}

/// # Errors
/// Returns an error if the tool registry or profile are invalid for the run spec.
#[allow(dead_code)]
pub fn build_stage_plan(
    run_spec: &RunSpec,
    registry: &ToolRegistry,
    profile: &Profile,
    run_id: RunId,
) -> Result<RunExecutionPlan> {
    build_run_execution_plan(run_spec, registry, profile, run_id)
}

fn run_layout_from_dir(base_dir: &Path) -> bijux_dna_runtime::run_layout::RunLayout {
    bijux_dna_runtime::run_layout::RunLayout {
        run_dir: base_dir.to_path_buf(),
        stages_dir: base_dir.join("stages"),
        summary_dir: base_dir.join("summary"),
        assessment_path: base_dir.join("input_assessment.json"),
        manifest_path: base_dir.join("execution_manifest.json"),
        environment_path: base_dir.join("environment.json"),
        metadata_path: base_dir.join("run_metadata.json"),
        events_path: base_dir.join("events.jsonl"),
    }
}
