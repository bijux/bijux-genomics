use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::args::{
    ExecuteRunRequest, ExecuteRunResult, PlanRunRequest, PlanRunResult, RenderReportRequest,
    RenderReportResult, RunRequest, RunResult,
};
use bijux_core::{build_execution_plan, ExecutionPlan, Profile, RunId, RunSpec, ToolRegistry};
use bijux_pipelines::registry::PipelineRegistry;
use bijux_pipelines::{Domain, PipelineProfile};

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
    let plan = build_execution_plan(request.run_spec, registry, request.profile, request.run_id)?;
    Ok(PlanRunResult { plan })
}

/// # Errors
/// Returns an error if execution fails.
pub fn execute_run(request: &ExecuteRunRequest) -> Result<ExecuteRunResult> {
    bijux_engine::primitives::execute_stage_plan(&request.plan, request.runner, None)?;
    Ok(ExecuteRunResult)
}

/// # Errors
/// Returns an error if report rendering fails.
pub fn render_report(request: &RenderReportRequest) -> Result<RenderReportResult> {
    let report_path = render_report_from_facts(&request.base_dir, &request.facts_path)?;
    Ok(RenderReportResult { report_path })
}

fn render_report_from_facts(base_dir: &Path, facts_path: &Path) -> Result<PathBuf> {
    let facts = bijux_analyze::load::load_facts(facts_path)?;
    let report_path = bijux_analyze::report::write_run_report_from_facts(base_dir, &facts)?;
    Ok(report_path)
}

/// # Errors
/// Returns an error if the tool registry or profile are invalid for the run spec.
pub fn build_stage_plan(
    run_spec: RunSpec,
    registry: &ToolRegistry,
    profile: Profile,
    run_id: RunId,
) -> Result<ExecutionPlan> {
    Ok(build_execution_plan(run_spec, registry, profile, run_id)?)
}
