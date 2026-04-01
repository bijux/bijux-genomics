use super::{
    anyhow, build_run_execution_plan, Domain, Path, PathBuf, PipelineProfile, PipelineRegistry,
    PlanRunRequest, PlanRunResult, Result, RunRequest, RunResult, ToolRegistry,
};

mod profile_selection;
mod planning_support;

/// Run execution mode for API pipeline execution.
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

pub use self::profile_selection::{select_pipeline, select_pipelines};

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
