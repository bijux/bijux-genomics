use super::{anyhow, Result, RunMode, RunRequest, RunResult};

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
