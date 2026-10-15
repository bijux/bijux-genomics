//! Compiler-cache lifetime, build authority and observed reuse.

use crate::commands::ops::tooling::artifact_env_with_common_test_env;
use crate::commands::ops::tooling::artifact_root_path;
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;
use anyhow::Context;
use anyhow::Result;
use serde::Serialize;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

#[cfg(unix)]
pub(super) const SCCACHE_UNIX_SOCKET_PATH_MAX_BYTES: usize = 107;

#[cfg(unix)]
pub(super) const SCCACHE_UNIX_RUNTIME_TMPDIR: &str = "/tmp";

#[cfg(unix)]
pub(super) const SCCACHE_RUSTC_WRAPPER: &str = "#!/bin/sh\ncompiler=$1\nshift\nif [ \"$#\" -eq 1 ] && [ \"$1\" = -vV ]; then\n  exec \"$compiler\" \"$@\"\nfi\nexec \"$BIJUX_CI_SCCACHE\" \"$compiler\" \"$@\"\n";

#[derive(Debug)]
pub(super) struct OwnedSccacheServer<'a> {
    pub(super) workspace: &'a Workspace,
    pub(super) wrapper: String,
    pub(super) runtime_env: Vec<(String, String)>,
}

impl Drop for OwnedSccacheServer<'_> {
    fn drop(&mut self) {
        let _ = ProcessRunner::new(self.workspace).run_owned_with_env(
            &self.wrapper,
            &["--stop-server".to_string()],
            &self.runtime_env,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FastBuildAuthority {
    pub(super) target: PathBuf,
    pub(super) shared: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct FastCacheStats {
    pub(super) cache_location: String,
    pub(super) compile_requests: u64,
    pub(super) requests_executed: u64,
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) cache_writes: u64,
    pub(super) cache_errors: u64,
    pub(super) cache_timeouts: u64,
    pub(super) cache_read_errors: u64,
    pub(super) cache_write_errors: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct FastCacheDelta {
    pub(super) cache_location: String,
    pub(super) compile_requests: u64,
    pub(super) requests_executed: u64,
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) cache_writes: u64,
    pub(super) cache_errors: u64,
    pub(super) cache_timeouts: u64,
    pub(super) cache_read_errors: u64,
    pub(super) cache_write_errors: u64,
    pub(super) hit_ratio_basis_points: u64,
}

pub(super) fn ci_fast_group_env(
    workspace: &Workspace,
    authority: &FastBuildAuthority,
    group_id: &str,
    sccache_runtime_env: &[(String, String)],
    group_compiler_env: &[(String, String)],
) -> Result<Vec<(String, String)>> {
    let mut envs = artifact_env_with_common_test_env(workspace)?;
    envs.push((
        "CARGO_TARGET_DIR".to_string(),
        authority.target.join(group_id).display().to_string(),
    ));
    envs.push(("CARGO_NET_OFFLINE".to_string(), "true".to_string()));
    envs.extend_from_slice(sccache_runtime_env);
    envs.extend_from_slice(group_compiler_env);
    Ok(envs)
}

pub(super) fn materialize_ci_fast_compiler_wrapper(
    report_root: &Path,
    compiler_wrapper: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let Some(wrapper) = compiler_wrapper.filter(|wrapper| {
        Path::new(wrapper).file_name().and_then(|name| name.to_str()) == Some("sccache")
    }) else {
        return Ok(Vec::new());
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let proxy = report_root.join("sccache-rustc-wrapper");
        bijux_dna_infra::write_bytes(&proxy, SCCACHE_RUSTC_WRAPPER.as_bytes())?;
        std::fs::set_permissions(&proxy, std::fs::Permissions::from_mode(0o700))?;
        Ok(vec![
            ("BIJUX_CI_SCCACHE".to_string(), wrapper.to_string()),
            ("RUSTC_WRAPPER".to_string(), proxy.display().to_string()),
        ])
    }

    #[cfg(not(unix))]
    {
        let _ = report_root;
        Ok(Vec::new())
    }
}

pub(super) fn resolve_ci_fast_sccache_runtime_env(
    compiler_wrapper: Option<&str>,
    configured_socket: Option<PathBuf>,
) -> Result<Vec<(String, String)>> {
    let uses_sccache = compiler_wrapper
        .and_then(|wrapper| Path::new(wrapper).file_name())
        .and_then(|name| name.to_str())
        == Some("sccache");
    if !uses_sccache {
        return Ok(Vec::new());
    }

    #[cfg(unix)]
    {
        let socket = configured_socket.unwrap_or_else(|| {
            PathBuf::from(format!("/tmp/bijux-sccache-{}.sock", std::process::id()))
        });
        if !socket.is_absolute()
            || socket
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        {
            anyhow::bail!("SCCACHE_SERVER_UDS must be absolute and normalized");
        }
        if socket.as_os_str().as_encoded_bytes().len() > SCCACHE_UNIX_SOCKET_PATH_MAX_BYTES {
            anyhow::bail!(
                "SCCACHE_SERVER_UDS exceeds the Unix socket path limit of \
                {SCCACHE_UNIX_SOCKET_PATH_MAX_BYTES} bytes"
            );
        }
        Ok(vec![
            ("SCCACHE_SERVER_UDS".to_string(), socket.display().to_string()),
            ("TMPDIR".to_string(), SCCACHE_UNIX_RUNTIME_TMPDIR.to_string()),
        ])
    }

    #[cfg(not(unix))]
    {
        let _ = configured_socket;
        Ok(Vec::new())
    }
}

pub(super) fn start_owned_sccache_server<'a>(
    workspace: &'a Workspace,
    compiler_wrapper: Option<&str>,
    runtime_env: &[(String, String)],
    manage_server: bool,
) -> Result<Option<OwnedSccacheServer<'a>>> {
    let Some(wrapper) = compiler_wrapper.filter(|wrapper| {
        Path::new(wrapper).file_name().and_then(|name| name.to_str()) == Some("sccache")
    }) else {
        return Ok(None);
    };
    if !manage_server {
        return Ok(None);
    }

    let output = ProcessRunner::new(workspace).run_owned_with_env(
        wrapper,
        &["--start-server".to_string()],
        runtime_env,
    )?;
    if !output.status.success() {
        anyhow::bail!(
            "start ci-fast sccache server: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(Some(OwnedSccacheServer {
        workspace,
        wrapper: wrapper.to_string(),
        runtime_env: runtime_env.to_vec(),
    }))
}

pub(super) fn verify_sccache_compiler(
    workspace: &Workspace,
    compiler_wrapper: Option<&str>,
    runtime_env: &[(String, String)],
) -> Result<()> {
    let Some(wrapper) = compiler_wrapper.filter(|wrapper| {
        Path::new(wrapper).file_name().and_then(|name| name.to_str()) == Some("sccache")
    }) else {
        return Ok(());
    };
    let output = ProcessRunner::new(workspace).run_owned_with_env(
        wrapper,
        &["rustc".to_string(), "-vV".to_string()],
        runtime_env,
    )?;
    if !output.status.success() {
        anyhow::bail!(
            "verify ci-fast sccache compiler handshake: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub(super) fn ci_fast_build_authority(workspace: &Workspace) -> Result<FastBuildAuthority> {
    resolve_ci_fast_build_authority_with_scratch_roots(
        workspace,
        std::env::var_os("BIJUX_CI_FAST_TARGET_DIR").map(PathBuf::from),
        &scheduler_scratch_roots(),
    )
}

#[cfg(test)]
pub(super) fn resolve_ci_fast_build_authority(
    workspace: &Workspace,
    configured: Option<PathBuf>,
) -> Result<FastBuildAuthority> {
    resolve_ci_fast_build_authority_with_scratch_roots(workspace, configured, &[])
}

pub(super) fn resolve_ci_fast_build_authority_with_scratch_roots(
    workspace: &Workspace,
    configured: Option<PathBuf>,
    scratch_roots: &[PathBuf],
) -> Result<FastBuildAuthority> {
    let shared = configured.is_some();
    let target = match configured {
        Some(target) => target,
        None => artifact_root_path(workspace)?.join("ci/ci-fast-target"),
    };
    if shared
        && (!target.is_absolute()
            || target
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::CurDir)))
    {
        anyhow::bail!("BIJUX_CI_FAST_TARGET_DIR must be absolute and normalized");
    }
    bijux_dna_infra::ensure_dir(&target)?;
    let canonical_target = target.canonicalize().context("canonicalize ci-fast build authority")?;
    if shared && canonical_target != target {
        anyhow::bail!("BIJUX_CI_FAST_TARGET_DIR must not traverse symbolic path aliases");
    }
    let target = canonical_target;
    if shared
        && scratch_roots.iter().any(|root| {
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            target.starts_with(canonical_root)
        })
    {
        anyhow::bail!(
            "BIJUX_CI_FAST_TARGET_DIR must not use scheduler-local scratch for warm-cache evidence"
        );
    }
    let source = workspace.root.canonicalize().context("canonicalize ci-fast source root")?;
    if shared
        && scratch_roots.iter().any(|root| {
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
            source.starts_with(canonical_root)
        })
    {
        anyhow::bail!(
            "ci-fast source workspace must not use scheduler-local scratch for warm-cache evidence"
        );
    }
    if shared && (target.starts_with(&source) || source.starts_with(&target)) {
        anyhow::bail!("BIJUX_CI_FAST_TARGET_DIR must be disjoint from the source workspace");
    }
    Ok(FastBuildAuthority { target, shared })
}

pub(super) fn scheduler_scratch_roots() -> Vec<PathBuf> {
    ["SNIC_TMP", "SLURM_TMPDIR"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
        .collect()
}

pub(super) fn rust_toolchain_version(workspace: &Workspace) -> Result<String> {
    let output = ProcessRunner::new(workspace).run(&["rustc", "--version"])?;
    if !output.status.success() {
        anyhow::bail!("resolve ci-fast Rust toolchain");
    }
    let version = String::from_utf8(output.stdout)?;
    version
        .split_whitespace()
        .nth(1)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .context("parse ci-fast Rust toolchain")
}

pub(super) fn compiler_wrapper_identity(workspace: &Workspace) -> Result<Option<String>> {
    let Some(wrapper) = compiler_wrapper_program(workspace)? else {
        return Ok(None);
    };
    let output = ProcessRunner::new(workspace).run(&[wrapper.as_str(), "--version"])?;
    if !output.status.success() {
        return Ok(None);
    }
    let version = String::from_utf8(output.stdout)?;
    Ok(parse_compiler_wrapper_identity(&version))
}

pub(super) fn compiler_wrapper_program(workspace: &Workspace) -> Result<Option<String>> {
    Ok(artifact_env_with_common_test_env(workspace)?
        .into_iter()
        .rev()
        .find_map(|(key, value)| (key == "RUSTC_WRAPPER").then_some(value))
        .filter(|value| !value.trim().is_empty()))
}

pub(super) fn parse_compiler_wrapper_identity(version: &str) -> Option<String> {
    version.split_whitespace().next().filter(|name| *name == "sccache").map(ToOwned::to_owned)
}

pub(super) fn read_sccache_stats(
    workspace: &Workspace,
    wrapper: &str,
    runtime_env: &[(String, String)],
) -> Result<Option<FastCacheStats>> {
    let output = ProcessRunner::new(workspace).run_owned_with_env(
        wrapper,
        &["--show-stats".to_string(), "--stats-format=json".to_string()],
        runtime_env,
    )?;
    if !output.status.success() {
        return Ok(None);
    }
    parse_sccache_stats(&serde_json::from_slice(&output.stdout)?).map(Some)
}

pub(super) fn parse_sccache_stats(value: &serde_json::Value) -> Result<FastCacheStats> {
    let stats = value.get("stats").unwrap_or(value);
    let count = |name: &str| -> Result<u64> {
        stats
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .with_context(|| format!("sccache statistics omit {name}"))
    };
    let grouped_count = |name: &str| -> Result<u64> {
        let group = stats.get(name).context("sccache statistics omit grouped counter")?;
        Ok(group
            .get("counts")
            .or_else(|| group.get("adv_counts"))
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flat_map(|counts| counts.values())
            .filter_map(serde_json::Value::as_u64)
            .sum())
    };
    Ok(FastCacheStats {
        cache_location: value
            .get("cache_location")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        compile_requests: count("compile_requests")?,
        requests_executed: count("requests_executed")?,
        cache_hits: grouped_count("cache_hits")?,
        cache_misses: grouped_count("cache_misses")?,
        cache_writes: count("cache_writes")?,
        cache_errors: grouped_count("cache_errors")?,
        cache_timeouts: count("cache_timeouts")?,
        cache_read_errors: count("cache_read_errors")?,
        cache_write_errors: count("cache_write_errors")?,
    })
}

pub(super) fn cache_stats_delta(
    before: Option<&FastCacheStats>,
    after: Option<&FastCacheStats>,
) -> Option<FastCacheDelta> {
    let (before, after) = (before?, after?);
    if before.cache_location.is_empty() || before.cache_location != after.cache_location {
        return None;
    }
    let delta = |end: u64, start: u64| end.checked_sub(start);
    let requests_executed = delta(after.requests_executed, before.requests_executed)?;
    let cache_hits = delta(after.cache_hits, before.cache_hits)?;
    let cache_misses = delta(after.cache_misses, before.cache_misses)?;
    let cacheable_requests = cache_hits.checked_add(cache_misses)?;
    let hit_ratio_basis_points =
        cache_hits.saturating_mul(10_000).checked_div(cacheable_requests).unwrap_or(0);
    Some(FastCacheDelta {
        cache_location: after.cache_location.clone(),
        compile_requests: delta(after.compile_requests, before.compile_requests)?,
        requests_executed,
        cache_hits,
        cache_misses,
        cache_writes: delta(after.cache_writes, before.cache_writes)?,
        cache_errors: delta(after.cache_errors, before.cache_errors)?,
        cache_timeouts: delta(after.cache_timeouts, before.cache_timeouts)?,
        cache_read_errors: delta(after.cache_read_errors, before.cache_read_errors)?,
        cache_write_errors: delta(after.cache_write_errors, before.cache_write_errors)?,
        hit_ratio_basis_points,
    })
}

pub(super) fn verified_warm_cache_mode(
    requested_cache_state: &str,
    shared_cache_authority: bool,
    shared_build_authority: bool,
    cache_delta: Option<&FastCacheDelta>,
    minimum_cache_hit_ratio_basis_points: u64,
) -> Option<&'static str> {
    if requested_cache_state != "warm" || !shared_cache_authority || !shared_build_authority {
        return None;
    }
    let delta = cache_delta?;
    if delta.cache_errors != 0
        || delta.cache_timeouts != 0
        || delta.cache_read_errors != 0
        || delta.cache_write_errors != 0
    {
        return None;
    }
    if delta.compile_requests == 0 && delta.requests_executed == 0 {
        return Some("build_target");
    }
    (delta.requests_executed > 0
        && delta.hit_ratio_basis_points >= minimum_cache_hit_ratio_basis_points)
        .then_some("compiler_cache")
}

#[cfg(test)]
mod tests {
    use super::cache_stats_delta;
    use super::ci_fast_group_env;
    use super::materialize_ci_fast_compiler_wrapper;
    use super::parse_compiler_wrapper_identity;
    use super::parse_sccache_stats;
    use super::resolve_ci_fast_build_authority;
    use super::resolve_ci_fast_build_authority_with_scratch_roots;
    use super::resolve_ci_fast_sccache_runtime_env;
    use super::start_owned_sccache_server;
    use super::verified_warm_cache_mode;
    use super::verify_sccache_compiler;
    use super::FastCacheStats;
    use crate::runtime::process::ProcessRunner;
    use crate::runtime::workspace::Workspace;

    #[test]
    fn fast_groups_isolate_targets_under_one_offline_build_authority() -> anyhow::Result<()> {
        let workspace = Workspace::resolve()?;
        let authority = resolve_ci_fast_build_authority(&workspace, None)?;
        let runtime_env = resolve_ci_fast_sccache_runtime_env(Some("sccache"), None)?;
        let first = ci_fast_group_env(&workspace, &authority, "metadata", &runtime_env, &[])?;
        let second = ci_fast_group_env(&workspace, &authority, "unit", &runtime_env, &[])?;

        assert!(first.iter().any(|(key, value)| key == "CARGO_NET_OFFLINE" && value == "true"));
        assert!(first.iter().any(|(key, value)| key == "TZ" && value == "UTC"));
        assert!(first.iter().any(|(key, value)| key == "LC_ALL" && value == "C"));
        let first_target = first
            .iter()
            .rev()
            .find(|(key, _)| key == "CARGO_TARGET_DIR")
            .map(|(_, value)| value.as_str());
        let second_target = second
            .iter()
            .rev()
            .find(|(key, _)| key == "CARGO_TARGET_DIR")
            .map(|(_, value)| value.as_str());
        assert_ne!(first_target, second_target);
        assert!(first_target.is_some_and(|value| value.ends_with("ci/ci-fast-target/metadata")));
        assert!(second_target.is_some_and(|value| value.ends_with("ci/ci-fast-target/unit")));
        assert!(first.iter().any(|(key, value)| {
            key == "SCCACHE_SERVER_UDS" && value.starts_with("/tmp/bijux-sccache-")
        }));
        assert!(first.iter().any(|(key, value)| key == "TMPDIR" && value == "/tmp"));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn sccache_transport_rejects_unix_socket_paths_that_cannot_bind() {
        let too_long = std::path::PathBuf::from(format!("/tmp/{}.sock", "x".repeat(108)));
        assert!(
            resolve_ci_fast_sccache_runtime_env(Some("/usr/bin/sccache"), Some(too_long)).is_err()
        );
        assert!(resolve_ci_fast_sccache_runtime_env(
            Some("/usr/bin/sccache"),
            Some(std::path::PathBuf::from("relative.sock"))
        )
        .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn sccache_transport_replaces_deep_inherited_runtime_tmpdir() -> anyhow::Result<()> {
        let workspace = Workspace::resolve()?;
        let authority = resolve_ci_fast_build_authority(&workspace, None)?;
        let runtime_env = resolve_ci_fast_sccache_runtime_env(
            Some("sccache"),
            Some(std::path::PathBuf::from("/tmp/bijux-sccache-test.sock")),
        )?;
        let envs = ci_fast_group_env(&workspace, &authority, "metadata", &runtime_env, &[])?;

        let tmpdirs = envs
            .iter()
            .filter(|(key, _)| key == "TMPDIR")
            .map(|(_, value)| value.as_str())
            .collect::<Vec<_>>();
        assert_eq!(tmpdirs, vec!["/tmp"]);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn ci_fast_owns_only_its_automatically_allocated_sccache_server() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let artifact_root = workspace.path("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let fixture = tempfile::tempdir_in(&artifact_root)?;
        let wrapper = fixture.path().join("sccache");
        let log = fixture.path().join("server-events.log");
        std::fs::write(
            &wrapper,
            "#!/bin/sh\nprintf '%s|%s\\n' \"$1\" \"$SCCACHE_SERVER_UDS\" >> \"$BIJUX_CI_TEST_SCCACHE_LOG\"\n",
        )?;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700))?;
        let runtime_env = vec![
            ("SCCACHE_SERVER_UDS".to_string(), "/tmp/bijux-sccache-test.sock".to_string()),
            ("BIJUX_CI_TEST_SCCACHE_LOG".to_string(), log.display().to_string()),
        ];

        let server = start_owned_sccache_server(&workspace, wrapper.to_str(), &runtime_env, true)?;
        assert!(server.is_some());
        verify_sccache_compiler(&workspace, wrapper.to_str(), &runtime_env)?;
        drop(server);
        assert_eq!(
            std::fs::read_to_string(&log)?,
            "--start-server|/tmp/bijux-sccache-test.sock\nrustc|/tmp/bijux-sccache-test.sock\n--stop-server|/tmp/bijux-sccache-test.sock\n"
        );

        std::fs::remove_file(&log)?;
        assert!(start_owned_sccache_server(&workspace, wrapper.to_str(), &runtime_env, false,)?
            .is_none());
        verify_sccache_compiler(&workspace, wrapper.to_str(), &runtime_env)?;
        assert_eq!(std::fs::read_to_string(&log)?, "rustc|/tmp/bijux-sccache-test.sock\n");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn ci_fast_bypasses_sccache_only_for_parallel_cargo_version_probes() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let artifact_root = workspace.path("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let fixture = tempfile::tempdir_in(&artifact_root)?;
        let compiler = fixture.path().join("rustc");
        let sccache = fixture.path().join("sccache");
        let log = fixture.path().join("wrapper-events.log");
        std::fs::write(
            &compiler,
            "#!/bin/sh\nprintf 'rustc|%s\\n' \"$*\" >> \"$BIJUX_CI_TEST_WRAPPER_LOG\"\n",
        )?;
        std::fs::write(
            &sccache,
            "#!/bin/sh\nprintf 'sccache|%s\\n' \"$*\" >> \"$BIJUX_CI_TEST_WRAPPER_LOG\"\n",
        )?;
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o700))?;
        std::fs::set_permissions(&sccache, std::fs::Permissions::from_mode(0o700))?;
        let envs = materialize_ci_fast_compiler_wrapper(fixture.path(), sccache.to_str())?;
        let Some(proxy) =
            envs.iter().find_map(|(key, value)| (key == "RUSTC_WRAPPER").then_some(value.as_str()))
        else {
            anyhow::bail!("generated sccache Rust compiler wrapper is missing");
        };
        let proxy = proxy.to_string();
        let mut invocation_env = envs;
        invocation_env.push(("BIJUX_CI_TEST_WRAPPER_LOG".to_string(), log.display().to_string()));

        let version = ProcessRunner::new(&workspace).run_owned_with_env(
            &proxy,
            &[compiler.display().to_string(), "-vV".to_string()],
            &invocation_env,
        )?;
        assert!(version.status.success());
        let compile = ProcessRunner::new(&workspace).run_owned_with_env(
            &proxy,
            &[compiler.display().to_string(), "--crate-name".to_string(), "example".to_string()],
            &invocation_env,
        )?;
        assert!(compile.status.success());
        assert_eq!(
            std::fs::read_to_string(log)?,
            format!("rustc|-vV\nsccache|{} --crate-name example\n", compiler.display())
        );
        Ok(())
    }

    #[test]
    fn explicit_fast_build_authority_is_stable_and_source_disjoint() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let root = root.path().canonicalize()?;
        let source = root.join("workspaces/commit/source");
        std::fs::create_dir_all(&source)?;
        let workspace = Workspace { root: source.clone() };
        let shared = root.join("shared/ci-fast-target");

        let authority = resolve_ci_fast_build_authority(&workspace, Some(shared))?;

        assert!(authority.shared);
        assert_eq!(authority.target, root.join("shared/ci-fast-target").canonicalize()?);
        assert!(resolve_ci_fast_build_authority(
            &workspace,
            Some(source.join("artifacts/ci-fast-target"))
        )
        .is_err());
        assert!(resolve_ci_fast_build_authority(
            &workspace,
            Some(std::path::PathBuf::from("relative-target"))
        )
        .is_err());

        let scheduler_scratch = root.join("scheduler-scratch");
        std::fs::create_dir_all(&scheduler_scratch)?;
        let rejected = resolve_ci_fast_build_authority_with_scratch_roots(
            &workspace,
            Some(scheduler_scratch.join("ci-fast-target")),
            std::slice::from_ref(&scheduler_scratch),
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("scheduler-local build authority was accepted"))?;
        assert!(rejected.to_string().contains("scheduler-local scratch"));

        let volatile_source = scheduler_scratch.join("source");
        std::fs::create_dir_all(&volatile_source)?;
        let volatile_workspace = Workspace { root: volatile_source };
        let rejected = resolve_ci_fast_build_authority_with_scratch_roots(
            &volatile_workspace,
            Some(root.join("shared/stable-target")),
            std::slice::from_ref(&scheduler_scratch),
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("scheduler-local source authority was accepted"))?;
        assert!(rejected.to_string().contains("source workspace"));
        Ok(())
    }

    #[test]
    fn sccache_delta_proves_hits_without_trusting_a_label() -> anyhow::Result<()> {
        let stats = |requests, executed, hits, misses, writes| FastCacheStats {
            cache_location: "Local disk: /shared/sccache".to_string(),
            compile_requests: requests,
            requests_executed: executed,
            cache_hits: hits,
            cache_misses: misses,
            cache_writes: writes,
            cache_errors: 0,
            cache_timeouts: 0,
            cache_read_errors: 0,
            cache_write_errors: 0,
        };
        let before = stats(20, 18, 12, 6, 6);
        let after = stats(30, 28, 20, 8, 8);
        let delta = cache_stats_delta(Some(&before), Some(&after))
            .ok_or_else(|| anyhow::anyhow!("cache delta was unavailable"))?;

        assert_eq!(delta.compile_requests, 10);
        assert_eq!(delta.requests_executed, 10);
        assert_eq!(delta.cache_hits, 8);
        assert_eq!(delta.cache_misses, 2);
        assert_eq!(delta.hit_ratio_basis_points, 8_000);
        assert!(cache_stats_delta(Some(&after), Some(&before)).is_none());
        Ok(())
    }

    #[test]
    fn warm_cache_evidence_accepts_compiler_hits_or_a_reused_build_target() {
        let delta =
            |compile_requests, requests_executed, hit_ratio_basis_points| super::FastCacheDelta {
                cache_location: "/shared/sccache".to_string(),
                compile_requests,
                requests_executed,
                cache_hits: requests_executed,
                cache_misses: 0,
                cache_writes: 0,
                cache_errors: 0,
                cache_timeouts: 0,
                cache_read_errors: 0,
                cache_write_errors: 0,
                hit_ratio_basis_points,
            };
        let compiler_cache = delta(10, 10, 10_000);
        let build_target = delta(0, 0, 0);
        let cold_compiler_cache = delta(10, 10, 0);

        assert_eq!(
            verified_warm_cache_mode("warm", true, true, Some(&compiler_cache), 8_000),
            Some("compiler_cache")
        );
        assert_eq!(
            verified_warm_cache_mode("warm", true, true, Some(&build_target), 8_000),
            Some("build_target")
        );
        assert_eq!(
            verified_warm_cache_mode("warm", true, true, Some(&cold_compiler_cache), 8_000),
            None
        );
        assert_eq!(verified_warm_cache_mode("warm", false, true, Some(&build_target), 8_000), None);
        assert_eq!(verified_warm_cache_mode("cold", true, true, Some(&build_target), 8_000), None);
    }

    #[test]
    fn sccache_json_parser_accepts_current_stats_envelope() -> anyhow::Result<()> {
        let stats = parse_sccache_stats(&serde_json::json!({
            "stats": {
                "compile_requests": 10,
                "requests_executed": 9,
                "cache_hits": {"counts": {"Rust": 7}, "adv_counts": {"rust": 7}},
                "cache_misses": {"counts": {"Rust": 2}, "adv_counts": {"rust": 2}},
                "cache_writes": 2,
                "cache_errors": {"counts": {}, "adv_counts": {}},
                "cache_timeouts": 0,
                "cache_read_errors": 0,
                "cache_write_errors": 0
            },
            "cache_location": "Local disk: /shared/sccache"
        }))?;

        assert_eq!(stats.cache_hits, 7);
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.cache_location, "Local disk: /shared/sccache");
        Ok(())
    }

    #[test]
    fn reference_wrapper_identity_requires_sccache_version_output() {
        assert_eq!(
            parse_compiler_wrapper_identity("sccache 0.10.0\n"),
            Some("sccache".to_string())
        );
        assert_eq!(parse_compiler_wrapper_identity("mock wrapper\n"), None);
        assert_eq!(parse_compiler_wrapper_identity(""), None);
    }
}
