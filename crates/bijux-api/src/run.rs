use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, PlanRunRequest, PlanRunResult,
    RenderReportRequest, RenderReportResult, RunRequest, RunResult, RunStatus,
};
use bijux_core::contract::{Profile, RunSpec, ToolRegistry};
use bijux_core::execution::execution_graph::ExecutionGraph;
use bijux_core::ids::RunId;
use bijux_engine::Engine;
use bijux_pipelines::registry::PipelineRegistry;
use bijux_pipelines::{Domain, PipelineProfile};
use bijux_runner::DockerRunner;
use bijux_stage_contract::{build_run_execution_plan, RunExecutionPlan};

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
    let profile = bijux_pipelines::registry::profile_by_id(request.domain, &request.profile_id)
        .map_err(|err| anyhow!("unknown pipeline profile {}: {err}", request.profile_id))?;
    bijux_infra::ensure_dir(&request.run_dir)?;
    let ledger_path = request.run_dir.join("defaults_ledger.json");
    let defaults = profile.defaults_ledger();
    bijux_infra::atomic_write_json(&ledger_path, &defaults)?;
    Ok(RunResult {
        run_dir: request.run_dir,
        profile_id: profile.id.to_string(),
    })
}

/// # Errors
/// Returns an error if the profile id is unknown.
pub fn select_pipeline(domain: Domain, profile_id: &str) -> Result<PipelineProfile> {
    bijux_pipelines::registry::profile_by_id(domain, profile_id)
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

/// # Errors
/// Returns an error if execution fails.
pub fn execute_run(request: &ExecuteRunRequest) -> Result<ExecuteRunResult> {
    let step = bijux_stage_contract::execution_step_from_stage_plan(&request.plan);
    bijux_runner::primitives::execute_step(&step, request.runner, None)?;
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
    let report_path = run_dir.join("run_artifacts").join("report.html");
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
                let actual = bijux_infra::hash_file_sha256(&path)?;
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
    let graph_path = base_dir.join("run_artifacts").join("graph.json");
    let graph_raw =
        std::fs::read_to_string(&graph_path).map_err(|err| anyhow!("read graph.json: {err}"))?;
    let graph: ExecutionGraph =
        serde_json::from_str(&graph_raw).map_err(|err| anyhow!("parse graph.json: {err}"))?;
    let runner = DockerRunner::new(None);
    let layout = run_layout_from_dir(base_dir);
    Engine::execute(&graph, &runner, &layout, None, None)?;
    Ok(())
}

/// # Errors
/// Returns an error if planning fails.
pub fn plan(request: PlanRequest) -> Result<PlanResponse> {
    let graph_hash = request.graph.hash()?;
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_core::contract::ContractVersion::v1(),
        "run_id": "plan-only",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": bijux_core::primitives::CacheKey::new("unknown", "unknown", "unknown", "unknown"),
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [],
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
    let (run_id, layout) = bijux_runtime::run_layout::create_run_layout(&request.run_dir)?;
    let runner: Box<dyn bijux_runtime::Runner> = match request.runner {
        bijux_environment::api::RunnerKind::Docker => Box::new(DockerRunner::new(None)),
        other => {
            return Err(anyhow!("runner {other} not supported for execute"));
        }
    };
    Engine::execute(&request.graph, runner.as_ref(), &layout, None, None)?;
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
    let graph_payload = bijux_core::primitives::to_canonical_json_bytes(&request.graph)?;
    bijux_infra::atomic_write_bytes(&graph_path, graph_payload.as_slice())?;
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_core::contract::ContractVersion::v1(),
        "run_id": "dry-run",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": bijux_core::primitives::CacheKey::new("unknown", "unknown", "unknown", "unknown"),
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [],
        "stages": [],
        "failures": [],
    });
    let manifest_path = request.run_dir.join("run_manifest.json");
    let payload = bijux_core::primitives::to_canonical_json_bytes(&manifest)?;
    bijux_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    Ok(DryRunResponse {
        graph_path,
        manifest_path,
    })
}

/// # Errors
/// Returns an error if policy checks fail or cannot be executed.
pub fn policy_audit() -> Result<serde_json::Value> {
    let workspace = std::env::current_dir()?;
    let mut guardrails = serde_json::Map::new();
    for crate_name in ["bijux-core", "bijux-engine", "bijux-api"] {
        let crate_root = workspace.join("crates").join(crate_name);
        let result = bijux_policies::check(
            &crate_root,
            &bijux_policies::GuardrailConfig::for_crate(crate_name),
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

fn render_report_from_facts(base_dir: &Path, facts_path: &Path) -> Result<PathBuf> {
    let facts = bijux_analyze::load::load_facts(facts_path)?;
    let report_path = bijux_analyze::report::write_run_report_from_facts(base_dir, &facts)?;
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

fn run_layout_from_dir(base_dir: &Path) -> bijux_runtime::run_layout::RunLayout {
    bijux_runtime::run_layout::RunLayout {
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
