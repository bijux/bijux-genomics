use std::path::{Component, Path};

use bijux_dna_core::contract::ExecutionStep;

use bijux_dna_environment::api::RuntimeKind;

pub(super) fn stage_workdir_in_container(out_dir: &Path, _runner: RuntimeKind) -> String {
    if let Ok(workdir) = std::env::var("BIJUX_STAGE_WORKDIR") {
        stage_workdir_for_value(out_dir, &workdir)
    } else {
        container_output_root().to_string()
    }
}

pub(super) fn stage_workdir_for_value(out_dir: &Path, workdir: &str) -> String {
    let Ok(relative) = Path::new(workdir).strip_prefix(out_dir) else {
        return container_output_root().to_string();
    };
    if relative.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) {
        return container_output_root().to_string();
    }
    if relative.as_os_str().is_empty() {
        return container_output_root().to_string();
    }
    format!("{}/{}", container_output_root(), relative.display())
}

fn container_output_root() -> &'static str {
    "/data/output"
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
