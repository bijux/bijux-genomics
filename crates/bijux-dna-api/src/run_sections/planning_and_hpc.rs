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

/// # Errors
/// Returns an error if the profile id is unknown.
pub fn select_pipeline(domain: Domain, profile_id: &str) -> Result<PipelineProfile> {
    bijux_dna_pipelines::registry::profile_by_id(domain, profile_id)
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

fn millis_u64(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn file_len_i64(len: u64) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

fn hpc_context_enabled() -> bool {
    std::env::var("BIJUX_RUN_CONTEXT")
        .map(|v| v.eq_ignore_ascii_case("hpc"))
        .unwrap_or(false)
}

fn enforce_hpc_results_layout(out_dir: &Path) -> Result<()> {
    let comps = out_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(mut idx) = comps
        .iter()
        .position(|v| v == "results" || v == "bijux-dna-results")
    else {
        return Err(anyhow!("HPC run out_dir must be under results root"));
    };
    if comps.get(idx).is_some_and(|v| v == "bijux-dna-results")
        && comps.get(idx + 1).is_some_and(|v| v == "results")
    {
        idx += 1;
    }
    if comps.len() < idx + 7 {
        return Err(anyhow!(
            "HPC out_dir must match results/<corpus>/<pipeline>/<stage>/<tool>/<timestamp>/<run_id>"
        ));
    }
    let ts = &comps[idx + 5];
    let ts_ok = regex::Regex::new(r"^\d{8}T\d{6}Z$")
        .map(|re| re.is_match(ts))
        .unwrap_or(false);
    if !ts_ok {
        return Err(anyhow!("HPC out_dir timestamp must match YYYYMMDDTHHMMSSZ"));
    }
    Ok(())
}

fn maybe_write_site_lock(out_dir: &Path) -> Result<()> {
    if !hpc_context_enabled() {
        return Ok(());
    }
    let comps = out_dir.components().collect::<Vec<_>>();
    let results_idx = comps.iter().position(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "bijux-dna-results" || s == "results"
    });
    let Some(idx) = results_idx else {
        return Ok(());
    };
    let mut root = PathBuf::new();
    for comp in &comps[..=idx] {
        root.push(comp.as_os_str());
    }
    let lock_path = root.join("site_lock.json");
    let apptainer_version = bijux_dna_environment::api::run_shell_capture("apptainer --version")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|v| !v.is_empty());
    let kernel = bijux_dna_environment::api::run_shell_capture("uname -r")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|v| !v.is_empty());
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|raw| {
            raw.lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|v| v.trim().to_string())
        });
    let payload = serde_json::json!({
        "schema_version": "bijux.site_lock.v1",
        "site": std::env::var("BIJUX_HPC_SITE").unwrap_or_else(|_| "lunarc".to_string()),
        "apptainer_version": apptainer_version,
        "kernel": kernel,
        "cpu_model": cpu_model,
    });
    bijux_dna_infra::atomic_write_json(&lock_path, &payload)?;
    Ok(())
}
