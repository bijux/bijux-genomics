//! Invocation-owned child environment; no process-global mutation.
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;

use super::super::invocation_policy::EffectiveRuntimePolicy;
use super::ToolInvocationRequest;

pub(super) fn invocation_environment(
    request: &ToolInvocationRequest,
    policy: &EffectiveRuntimePolicy,
    temporary: &Path,
    cache: &Path,
    home: &Path,
    work: &Path,
) -> BTreeMap<OsString, OsString> {
    let mut environment = std::env::vars_os().collect::<BTreeMap<_, _>>();
    for (key, value) in [
        ("LC_ALL", policy.deterministic_env.lc_all.as_deref().unwrap_or("C")),
        ("LANG", policy.deterministic_env.lang.as_deref().unwrap_or("C")),
        ("TZ", policy.deterministic_env.tz.as_deref().unwrap_or("UTC")),
    ] {
        environment.insert(OsString::from(key), OsString::from(value));
    }
    for (key, value) in [
        ("PATH", policy.deterministic_env.path.clone()),
        ("BIJUX_UMASK", policy.deterministic_env.umask.clone()),
        ("BIJUX_STAGE_MEMORY_MB", policy.memory_mb.map(|value| value.to_string())),
        ("BIJUX_COMPRESSION_THREADS", policy.compression_threads.map(|value| value.to_string())),
        ("BIJUX_STAGE_SEED", request.context.seed.map(|value| value.to_string())),
    ] {
        if let Some(value) = value {
            environment.insert(OsString::from(key), OsString::from(value));
        }
    }
    environment
        .insert(OsString::from("BIJUX_STAGE_THREADS"), OsString::from(policy.threads.to_string()));
    for (key, path) in [
        ("TMPDIR", temporary),
        ("HOME", home),
        ("XDG_CACHE_HOME", cache),
        ("BIJUX_CACHE_ROOT", cache),
        ("BIJUX_STAGE_WORKDIR", work),
    ] {
        environment.insert(OsString::from(key), path.as_os_str().to_os_string());
    }
    environment
}
