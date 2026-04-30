use super::{
    build_run_execution_plan, Path, Profile, Result, RunExecutionPlan, RunId, RunSpec, ToolRegistry,
};

mod dry_run;
mod execute_run;
mod lifecycle;
mod plan_response;
mod planner_manifest_support;
mod rendering;
mod replay;
mod status;
mod summary_artifact;
mod workspace_audit;

pub use dry_run::dry_run;
pub use execute_run::execute;
pub use plan_response::plan;
pub use rendering::{execute_and_report, render_report};
pub use replay::replay_manifest;
pub use status::status;
pub use workspace_audit::{policy_audit, workspace_edges, write_workspace_audit};

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
