use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::args::{
    ExecuteRunRequest, ExecuteRunResult, PlanRunRequest, PlanRunResult, RenderReportRequest,
    RenderReportResult, RunRequest, RunResult, RunStatus,
};
use bijux_core::contract::{Profile, RunSpec, ToolRegistry};
use bijux_core::ids::RunId;
use bijux_core::plan::execution_graph::ExecutionGraph;
use bijux_engine::RuntimeServices;
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
    let services = RuntimeServices { runner: &runner };
    bijux_engine::execute(&graph, &services)?;
    Ok(())
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
