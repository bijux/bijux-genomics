use super::{anyhow, Path, PathBuf, Result};

pub(super) fn millis_u64(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

pub(super) fn file_len_i64(len: u64) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

pub(super) fn hpc_context_enabled() -> bool {
    std::env::var("BIJUX_RUN_CONTEXT").map(|v| v.eq_ignore_ascii_case("hpc")).unwrap_or(false)
}

pub(super) fn enforce_hpc_results_layout(out_dir: &Path) -> Result<()> {
    let comps = out_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(idx) = comps.iter().position(|v| v == "results") else {
        return Err(anyhow!("HPC run out_dir must be under results root"));
    };
    if comps.len() < idx + 7 {
        return Err(anyhow!(
            "HPC out_dir must match results/<corpus>/<pipeline>/<stage>/<tool>/<timestamp>/<run_id>"
        ));
    }
    let ts = &comps[idx + 5];
    let ts_ok = regex::Regex::new(r"^\d{8}T\d{6}Z$").map(|re| re.is_match(ts)).unwrap_or(false);
    if !ts_ok {
        return Err(anyhow!("HPC out_dir timestamp must match YYYYMMDDTHHMMSSZ"));
    }
    Ok(())
}

pub(super) fn maybe_write_site_lock(out_dir: &Path) -> Result<()> {
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
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|raw| {
        raw.lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(':').nth(1))
            .map(|v| v.trim().to_string())
    });
    let payload = serde_json::json!({
        "schema_version": "bijux.site_lock.v1",
        "site": resolved_site_name()?,
        "apptainer_version": apptainer_version,
        "kernel": kernel,
        "cpu_model": cpu_model,
    });
    bijux_dna_infra::atomic_write_json(&lock_path, &payload)?;
    Ok(())
}

fn resolved_site_name_with<F>(lookup: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup("BIJUX_HPC_SITE")
        .ok_or_else(|| anyhow!("BIJUX_HPC_SITE must be declared for HPC site locks"))
}

fn resolved_site_name() -> Result<String> {
    resolved_site_name_with(env_value)
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{enforce_hpc_results_layout, resolved_site_name_with};
    use std::path::Path;

    #[test]
    fn resolved_site_name_prefers_explicit_hpc_site() {
        let lookup = |key: &str| match key {
            "BIJUX_HPC_SITE" => Some("cluster-a".to_string()),
            "BIJUX_PLATFORM" => Some("platform-b".to_string()),
            "HOSTNAME" => Some("node-01.example".to_string()),
            _ => None,
        };
        let resolved = match resolved_site_name_with(lookup) {
            Ok(value) => value,
            Err(error) => panic!("site lookup should succeed: {error}"),
        };
        assert_eq!(resolved, "cluster-a");
    }

    #[test]
    fn resolved_site_name_requires_explicit_hpc_site() {
        let lookup = |key: &str| match key {
            "BIJUX_PLATFORM" => Some("apptainer-amd64".to_string()),
            "HOSTNAME" => Some("node-01.example".to_string()),
            _ => None,
        };
        let error = match resolved_site_name_with(lookup) {
            Ok(value) => panic!("missing BIJUX_HPC_SITE must fail, got {value}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("BIJUX_HPC_SITE must be declared for HPC site locks"));
    }

    #[test]
    fn hpc_results_layout_rejects_legacy_results_root_name() {
        let path = Path::new(
            "/hpc/root/bijux-dna-results/corpus-a/pipeline-x/stage-y/tool-z/20260211T120001Z/run-123",
        );
        let error = match enforce_hpc_results_layout(path) {
            Ok(()) => panic!("legacy root must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("HPC run out_dir must be under results root"));
    }
}
