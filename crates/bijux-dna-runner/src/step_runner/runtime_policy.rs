use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path};
use std::time::Duration;

use bijux_dna_core::contract::ExecutionStep;

use bijux_dna_containers::api::RuntimeKind;

#[derive(Clone, Copy)]
pub(super) struct StepRuntimeContext<'a> {
    pub(super) timeout: Option<Duration>,
    pub(super) executable: Option<&'a Path>,
    pub(super) environment: Option<&'a BTreeMap<OsString, OsString>>,
}

pub(super) fn environment_value(
    environment: Option<&BTreeMap<OsString, OsString>>,
    key: &str,
) -> Option<String> {
    match environment.and_then(|values| values.get(OsStr::new(key))) {
        // A non-text owned value shadows the ambient value, as env::var did.
        Some(value) => value.to_str().map(str::to_owned),
        None => std::env::var(key).ok(),
    }
}

pub(super) fn stage_workdir_in_container(
    out_dir: &Path,
    _runner: RuntimeKind,
    environment: Option<&BTreeMap<OsString, OsString>>,
) -> String {
    if let Some(workdir) = environment_value(environment, "BIJUX_STAGE_WORKDIR") {
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

pub(super) fn runtime_env_exports(
    environment: Option<&BTreeMap<OsString, OsString>>,
) -> Vec<(String, String)> {
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
        if let Some(value) = environment_value(environment, key) {
            pairs.push((key.to_string(), value));
        }
    }
    pairs
}

pub(super) fn configured_memory_mb(
    step: &ExecutionStep,
    environment: Option<&BTreeMap<OsString, OsString>>,
) -> f64 {
    if let Some(value) = environment_value(environment, "BIJUX_STAGE_MEMORY_MB") {
        if let Ok(parsed) = value.parse::<f64>() {
            if parsed.is_finite() && parsed > 0.0 {
                return parsed;
            }
        }
    }
    f64::from(step.resources.mem_gb.max(1)) * 1024.0
}

pub(super) fn network_allowed() -> bool {
    network_allowed_with_context(None)
}

pub(super) fn network_allowed_with_context(
    environment: Option<&BTreeMap<OsString, OsString>>,
) -> bool {
    environment_value(environment, "BIJUX_ALLOW_NETWORK")
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}
