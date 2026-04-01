use super::{
    anyhow, build_run_execution_plan, ensure_stage_supported_by_runner, DockerRunner,
    ExecuteRequest, ExecuteResponse, Path, Profile, Result, RunExecutionPlan, RunId, RunSpec,
    RunnerContractKind, ToolRegistry,
};
use bijux_dna_engine::Engine;

mod dry_run;
mod lifecycle;
mod plan_response;
mod rendering;
mod replay;
mod status;
mod summary_artifact;
mod workspace_audit;

pub use dry_run::dry_run;
pub use plan_response::plan;
pub use rendering::{execute_and_report, render_report};
pub use replay::replay_manifest;
pub use status::status;
pub use workspace_audit::{policy_audit, workspace_edges, write_workspace_audit};

/// # Errors
/// Returns an error if execution fails.
pub fn execute(request: &ExecuteRequest) -> Result<ExecuteResponse> {
    let runner_contract = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Docker => RunnerContractKind::Docker,
        other => return Err(anyhow!("runner {other} not supported for execute")),
    };
    for step in request.graph.steps() {
        ensure_stage_supported_by_runner(runner_contract, step.stage_id.as_str())?;
    }
    let (run_id, layout) = bijux_dna_runtime::run_layout::create_run_layout(&request.run_dir)?;
    let runner: Box<dyn bijux_dna_runtime::Runner> = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Docker => Box::new(DockerRunner::new(None)),
        other => {
            return Err(anyhow!("runner {other} not supported for execute"));
        }
    };
    Engine::default().execute(&request.graph, runner.as_ref(), &layout, None, None)?;
    let summary_path = layout.summary_dir.join("run_summary.json");
    summary_artifact::write_run_summary_artifact(
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
