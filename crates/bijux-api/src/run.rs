use anyhow::{anyhow, Result};

use crate::args::{RunRequest, RunResult};
use bijux_core::{build_execution_plan, ExecutionPlan, Profile, RunId, RunSpec, ToolRegistry};

#[derive(Debug, Clone, Copy)]
pub enum RunMode {
    PlanOnly,
    Execute,
}

/// # Errors
/// Returns an error if the profile id is unknown or IO setup fails.
pub fn run_pipeline(request: RunRequest, _mode: RunMode) -> Result<RunResult> {
    let profile = bijux_pipelines::registry::profile_by_id(request.domain, &request.profile_id)
        .map_err(|err| anyhow!("unknown pipeline profile {}: {err}", request.profile_id))?;
    bijux_io::ensure_dir(&request.run_dir)?;
    Ok(RunResult {
        run_dir: request.run_dir,
        profile_id: profile.id.to_string(),
    })
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
