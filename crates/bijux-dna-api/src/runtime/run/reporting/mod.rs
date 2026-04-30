use super::{
    build_run_execution_plan, Path, Profile, Result, RunExecutionPlan, RunId, RunSpec, ToolRegistry,
};

mod dry_run;
mod cache_explain;
mod environment_identity;
mod evidence_support;
mod failure_injection;
mod execute_run;
mod lifecycle;
mod local_workflows;
mod operations;
mod operator_controls;
mod plan_response;
mod planner_manifest_support;
mod rendering;
mod replay;
mod replay_failed;
mod replay_success;
mod status;
mod summary_artifact;
mod workspace_audit;

pub use dry_run::dry_run;
pub use cache_explain::explain_cache_hit_miss;
pub use environment_identity::environment_identity;
pub use execute_run::execute;
pub use failure_injection::run_local_failure_injection;
pub use local_workflows::{
    execute_local_bam_workflow, execute_local_fastq_workflow, execute_local_vcf_workflow,
};
pub use operator_controls::{cancel_run, operator_health, pause_run, resume_run};
pub use plan_response::plan;
pub use rendering::{execute_and_report, render_report};
pub use replay::replay_manifest;
pub use replay_failed::{assess_failed_replay_eligibility, replay_failed_run};
pub use replay_success::explain_successful_replay;
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
