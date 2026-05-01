use super::{
    anyhow, build_run_execution_plan, Domain, Path, PathBuf, PlanRunRequest, PlanRunResult, Result,
    RunRequest, RunResult, ToolRegistry,
};

mod planning_support;
mod profile_selection;
mod run_bootstrap;

/// Run execution mode for API pipeline execution.
///
/// Stability: v1 (stable).
pub enum RunMode {
    PlanOnly,
    Execute,
}

pub use self::run_bootstrap::run_pipeline;

pub use self::planning_support::{
    explain_pipeline_profile, stage_external_asset_requirement, stage_requires_local_assets,
    validate_pipeline_profile, StageAssetClass, StageExternalAssetRequirement,
};
pub use self::profile_selection::{select_pipeline, select_pipelines};

/// # Errors
/// Returns an error if planning fails for the requested run.
pub fn plan_run(request: PlanRunRequest, registry: &ToolRegistry) -> Result<PlanRunResult> {
    let plan =
        build_run_execution_plan(&request.run_spec, registry, &request.profile, request.run_id)?;
    Ok(PlanRunResult { plan })
}

/// # Errors
/// Returns an error if planning fails for the requested run.
pub fn plan_only(request: PlanRunRequest, registry: &ToolRegistry) -> Result<PlanRunResult> {
    plan_run(request, registry)
}

pub(super) fn millis_u64(elapsed: std::time::Duration) -> u64 {
    planning_support::millis_u64(elapsed)
}

pub(super) fn file_len_i64(len: u64) -> i64 {
    planning_support::file_len_i64(len)
}

pub(super) fn hpc_context_enabled() -> bool {
    planning_support::hpc_context_enabled()
}

pub(super) fn enforce_hpc_results_layout(out_dir: &Path) -> Result<()> {
    planning_support::enforce_hpc_results_layout(out_dir)
}

pub(super) fn maybe_write_site_lock(out_dir: &Path) -> Result<()> {
    planning_support::maybe_write_site_lock(out_dir)
}
