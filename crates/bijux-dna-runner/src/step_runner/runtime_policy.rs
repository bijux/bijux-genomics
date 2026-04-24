use std::path::Path;

use bijux_dna_core::contract::ExecutionStep;

use bijux_dna_environment::api::RuntimeKind;

pub(super) fn stage_workdir_in_container(out_dir: &Path, _runner: RuntimeKind) -> String {
    let output_root = "/data/output";
    if let Ok(workdir) = std::env::var("BIJUX_STAGE_WORKDIR") {
        let out_dir_prefix = format!("{}/", out_dir.display());
        if workdir.starts_with(&out_dir_prefix) {
            format!("{output_root}/{}", workdir.trim_start_matches(&out_dir_prefix))
        } else {
            output_root.to_string()
        }
    } else {
        output_root.to_string()
    }
}

pub(super) fn runtime_env_exports() -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for key in [
        "LC_ALL",
        "LANG",
        "TZ",
        "TMPDIR",
        "HOME",
        "XDG_CACHE_HOME",
        "BIJUX_CACHE_ROOT",
        "BIJUX_STAGE_THREADS",
        "BIJUX_STAGE_MEMORY_MB",
        "BIJUX_COMPRESSION_THREADS",
        "BIJUX_STAGE_SEED",
        "BIJUX_UMASK",
    ] {
        if let Ok(value) = std::env::var(key) {
            pairs.push((key.to_string(), value));
        }
    }
    pairs
}

pub(super) fn configured_memory_mb(step: &ExecutionStep) -> f64 {
    if let Ok(value) = std::env::var("BIJUX_STAGE_MEMORY_MB") {
        if let Ok(parsed) = value.parse::<f64>() {
            if parsed.is_finite() && parsed > 0.0 {
                return parsed;
            }
        }
    }
    f64::from(step.resources.mem_gb.max(1)) * 1024.0
}

pub(super) fn network_allowed() -> bool {
    std::env::var("BIJUX_ALLOW_NETWORK")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}
