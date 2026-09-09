use super::super::suite_selection::governed_slow_expression;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use bijux_dna_policies::{validate_ci_build_authority, CiBuildAuthority};
use serde::{Deserialize, Serialize};

use super::{
    artifact_env, artifact_env_with_common_test_env, artifact_root_path, ci_test_env,
    ensure_help_only, env_or_default, fs, merge_outcomes, read_coverage_runner_flag,
    resolved_nextest_expression, resolved_nextest_profile, resolved_nextest_threads,
    resolved_run_ignored, run_check_ids, run_make_target, run_program_with_env,
    set_assets_readonly, write_json_pretty, Context, OpsCommandOutcome, Result, Workspace,
};
use crate::application::ci::{
    CompiledWorkspaceExecutables, CoverageAttestation, ExpectedTestInventory, NextestReuseInputs,
    ShardDiscoveryPlan,
};
use crate::runtime::compilation_cache::CompilationCache;
use crate::runtime::process::{ParallelProcessSpec, ProcessRunner};
use crate::runtime::source_snapshot::TrackedSourceSnapshot;

const FULL_SHARDS_SCHEMA_V5: &str = "bijux.ci.full_shards.v5";
const FAST_GROUPS_SCHEMA_V3: &str = "bijux.ci.fast_groups.v3";
const FAST_TIMING_SCHEMA_V5: &str = "bijux.ci.fast_timing.v5";
const CI_CRITICAL_PATH_BASELINE_SCHEMA_V1: &str = "bijux.ci.critical_path_baseline.v1";
const FAST_TIMING_HISTORY_SCHEMA_V4: &str = "bijux.ci.fast_timing_history.v4";
const FULL_SHARD_SCRATCH_ROOT_ENV: &str = "BIJUX_CI_FULL_SHARD_SCRATCH_ROOT";
#[cfg(unix)]
const SCCACHE_UNIX_SOCKET_PATH_MAX_BYTES: usize = 107;
#[cfg(unix)]
const SCCACHE_UNIX_RUNTIME_TMPDIR: &str = "/tmp";
#[cfg(unix)]
const SCCACHE_RUSTC_WRAPPER: &str = "#!/bin/sh\ncompiler=$1\nshift\nif [ \"$#\" -eq 1 ] && [ \"$1\" = -vV ]; then\n  exec \"$compiler\" \"$@\"\nfi\nexec \"$BIJUX_CI_SCCACHE\" \"$compiler\" \"$@\"\n";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FullShardContract {
    schema_version: String,
    coverage: FullShardCoverage,
    partition_strategy: String,
    workspace_isolation: String,
    build_authority: String,
    runtime_dependency_report: String,
    runtime_dependencies: Vec<FullShardRuntimeDependency>,
    archive_build_jobs: usize,
    archive_root: String,
    shard_artifact_root: String,
    report_root: String,
    shards: Vec<FullShard>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FullShardRuntimeDependency {
    id: String,
    executable: String,
    version_args: Vec<String>,
    expected_version_line: String,
    provisioning: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FullShardCoverage {
    workspace: bool,
    all_targets: bool,
    all_features: bool,
    run_ignored: String,
    overlap_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FullShard {
    id: String,
    partition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FullShardRoots {
    artifact_root: PathBuf,
    cargo_target: PathBuf,
    test_target: PathBuf,
    coverage_target: PathBuf,
    test_tmp: PathBuf,
    coverage_tmp: PathBuf,
    test_profraw: PathBuf,
    coverage_profraw: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FullArchiveRoots {
    artifact_root: PathBuf,
    archive_extract: PathBuf,
    cargo_target: PathBuf,
    archive_file: PathBuf,
    build_log: PathBuf,
}

#[derive(Debug)]
struct OwnedSccacheServer<'a> {
    workspace: &'a Workspace,
    wrapper: String,
    runtime_env: Vec<(String, String)>,
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
struct FullShardStorageRoots {
    retained_report_root: PathBuf,
    scratch_root: PathBuf,
}

struct FullShardEphemera {
    scratch_workspaces: PathBuf,
    archive_root: PathBuf,
    shard_artifact_roots: Vec<PathBuf>,
    armed: bool,
}

impl FullShardEphemera {
    fn new(
        scratch_root: &Path,
        archive_roots: &FullArchiveRoots,
        shard_roots: &[FullShardRoots],
    ) -> Self {
        Self {
            scratch_workspaces: scratch_root.join("workspaces"),
            archive_root: archive_roots.artifact_root.clone(),
            shard_artifact_roots: shard_roots
                .iter()
                .map(|roots| roots.artifact_root.clone())
                .collect(),
            armed: true,
        }
    }

    fn retain_bounded_evidence(&mut self) -> Result<()> {
        remove_full_shard_ephemera(&self.scratch_workspaces)?;
        for path in &self.shard_artifact_roots {
            remove_full_shard_ephemera(path)?;
        }
        remove_full_shard_ephemera(&self.archive_root)?;
        self.armed = false;
        Ok(())
    }
}

impl Drop for FullShardEphemera {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.retain_bounded_evidence();
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastGroupContract {
    schema_version: String,
    max_parallelism: usize,
    warm_cache_target_milliseconds: u64,
    minimum_cache_hit_ratio_basis_points: u64,
    history_limit: usize,
    report_root: String,
    timing_report: String,
    timing_history: String,
    baseline_report: String,
    reference_runner: FastReferenceRunner,
    excluded_lanes: Vec<FastExcludedLane>,
    groups: Vec<FastGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FastExcludedLane {
    id: String,
    reason: String,
    evidence_command: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastReferenceRunner {
    os: String,
    architecture: String,
    rust_toolchain: String,
    compiler_wrapper: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastGroup {
    id: String,
    dependencies: Vec<String>,
    program: String,
    args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FastTimingHistory {
    schema_version: String,
    observations: Vec<FastTimingObservation>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FastTimingObservation {
    observed_at_utc: String,
    source_commit: String,
    os: String,
    architecture: String,
    #[serde(default)]
    rust_toolchain: String,
    #[serde(default)]
    compiler_wrapper: String,
    cache_state: String,
    #[serde(default)]
    cache_authority: String,
    #[serde(default)]
    build_authority: String,
    #[serde(default)]
    cache_hit_ratio_basis_points: u64,
    #[serde(default)]
    cache_reuse_mode: String,
    wall_time_milliseconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FastBuildAuthority {
    target: PathBuf,
    shared: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FastCacheStats {
    cache_location: String,
    compile_requests: u64,
    requests_executed: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_writes: u64,
    cache_errors: u64,
    cache_timeouts: u64,
    cache_read_errors: u64,
    cache_write_errors: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FastCacheDelta {
    cache_location: String,
    compile_requests: u64,
    requests_executed: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_writes: u64,
    cache_errors: u64,
    cache_timeouts: u64,
    cache_read_errors: u64,
    cache_write_errors: u64,
    hit_ratio_basis_points: u64,
}

pub(in super::super) fn tooling_ci_fmt(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-fmt", args)?;
    let envs = artifact_env(workspace)?;
    run_program_with_env(
        workspace,
        "cargo",
        &["fmt".to_string(), "--all".to_string(), "--".to_string(), "--check".to_string()],
        &envs,
    )
}

pub(in super::super) fn tooling_ci_clippy(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-clippy", args)?;
    let mut envs = artifact_env(workspace)?;
    envs.push(("CLIPPY_CONF_DIR".to_string(), "configs/rust".to_string()));
    if let Ok(value) = std::env::var("CARGO_BUILD_JOBS") {
        if !value.trim().is_empty() {
            envs.push(("CARGO_BUILD_JOBS".to_string(), value));
        }
    }
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "clippy".to_string(),
            "--workspace".to_string(),
            "--all-targets".to_string(),
            "--all-features".to_string(),
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ],
        &envs,
    )
}

pub(in super::super) fn tooling_ci_clippy_executors(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-clippy-executors", args)?;
    let mut envs = artifact_env(workspace)?;
    if let Ok(value) = std::env::var("CARGO_BUILD_JOBS") {
        if !value.trim().is_empty() {
            envs.push(("CARGO_BUILD_JOBS".to_string(), value));
        }
    }
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "clippy".to_string(),
            "--all-targets".to_string(),
            "--all-features".to_string(),
            "-p".to_string(),
            "bijux-dna-engine".to_string(),
            "-p".to_string(),
            "bijux-dna-runner".to_string(),
            "-p".to_string(),
            "bijux-dna-runtime".to_string(),
            "-p".to_string(),
            "bijux-dna-api".to_string(),
            "-p".to_string(),
            "bijux-dna-stages-bam".to_string(),
            "-p".to_string(),
            "bijux-dna-stages-vcf".to_string(),
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ],
        &envs,
    )
}

pub(in super::super) fn tooling_ci_audit(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-audit", args)?;
    let mut stdout = String::new();
    run_check_ids(&mut stdout, &["check-audit-allowlist", "check-deny-policy-deviations"])?;
    let outcome = run_program_with_env(
        workspace,
        "cargo",
        &[
            "deny".to_string(),
            "check".to_string(),
            "--config".to_string(),
            "configs/rust/deny.toml".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome))
}

pub(in super::super) fn tooling_ci_install_tools(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-install-tools", args)?;
    let authority = read_ci_build_authority(&workspace.path("configs/ci/build-authority.json"))?;
    let executor = run_program_with_env(
        workspace,
        "cargo",
        &[
            "install".to_string(),
            "--locked".to_string(),
            authority.test_executor.package,
            "--version".to_string(),
            authority.test_executor.version,
        ],
        &artifact_env(workspace)?,
    )?;
    let supporting_tools = run_program_with_env(
        workspace,
        "cargo",
        &[
            "install".to_string(),
            "--locked".to_string(),
            "cargo-llvm-cov".to_string(),
            "cargo-deny".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    Ok(merge_outcomes(executor, supporting_tools))
}

pub(in super::super) fn tooling_ci_fast(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-fast", args)?;
    let contract_path = workspace.path("configs/ci/fast-groups.json");
    let contract: FastGroupContract = serde_json::from_slice(&fs::read(&contract_path)?)
        .with_context(|| format!("parse {}", contract_path.display()))?;
    validate_fast_group_contract(&contract)?;

    let report_root = ci_output_path(workspace, &contract.report_root)?;
    bijux_dna_infra::ensure_dir(&report_root)?;
    let build_authority = ci_fast_build_authority(workspace)?;
    let compiler_wrapper = compiler_wrapper_program(workspace)?;
    let configured_sccache_socket = std::env::var_os("SCCACHE_SERVER_UDS").map(PathBuf::from);
    let sccache_runtime_env = resolve_ci_fast_sccache_runtime_env(
        compiler_wrapper.as_deref(),
        configured_sccache_socket.clone(),
    )?;
    let _owned_sccache_server = start_owned_sccache_server(
        workspace,
        compiler_wrapper.as_deref(),
        &sccache_runtime_env,
        configured_sccache_socket.is_none(),
    )?;
    verify_sccache_compiler(workspace, compiler_wrapper.as_deref(), &sccache_runtime_env)?;
    let group_compiler_env =
        materialize_ci_fast_compiler_wrapper(&report_root, compiler_wrapper.as_deref())?;
    let cache_before = compiler_wrapper.as_deref().and_then(|wrapper| {
        read_sccache_stats(workspace, wrapper, &sccache_runtime_env).ok().flatten()
    });
    let started = Instant::now();
    let mut pending =
        contract.groups.iter().map(|group| group.id.as_str()).collect::<BTreeSet<_>>();
    let mut passed = BTreeSet::new();
    let mut failed = BTreeSet::new();
    let mut receipts = Vec::with_capacity(contract.groups.len());
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut group_durations = BTreeMap::new();
    let mut compiler_lock_wait_events = 0usize;
    let mut compiler_lock_wait_duration_upper_bound_milliseconds = 0u64;
    let mut compiler_lock_wait_targets = BTreeSet::new();

    while !pending.is_empty() {
        let blocked = contract
            .groups
            .iter()
            .filter(|group| {
                pending.contains(group.id.as_str())
                    && group
                        .dependencies
                        .iter()
                        .any(|dependency| failed.contains(dependency.as_str()))
            })
            .collect::<Vec<_>>();
        for group in blocked {
            pending.remove(group.id.as_str());
            failed.insert(group.id.as_str());
            group_durations.insert(group.id.clone(), 0);
            stderr.push_str(&format!("ci-fast group {}: blocked by failed dependency\n", group.id));
            receipts.push(serde_json::json!({
                "id": group.id,
                "status": "blocked_dependency",
                "dependencies": group.dependencies,
                "command": std::iter::once(group.program.as_str())
                    .chain(group.args.iter().map(String::as_str))
                    .collect::<Vec<_>>(),
                "duration_milliseconds": 0,
                "compiler_lock_wait_events": 0,
                "build_target": build_authority.target.join(&group.id),
                "exit_code": 1,
                "log": serde_json::Value::Null,
            }));
        }
        if pending.is_empty() {
            break;
        }

        let ready = contract
            .groups
            .iter()
            .filter(|group| {
                pending.contains(group.id.as_str())
                    && group
                        .dependencies
                        .iter()
                        .all(|dependency| passed.contains(dependency.as_str()))
            })
            .take(contract.max_parallelism)
            .collect::<Vec<_>>();
        if ready.is_empty() {
            anyhow::bail!("ci-fast DAG has no ready group");
        }
        let specs = ready
            .iter()
            .map(|group| {
                Ok(ParallelProcessSpec {
                    label: group.id.clone(),
                    program: group.program.clone(),
                    args: group.args.clone(),
                    envs: ci_fast_group_env(
                        workspace,
                        &build_authority,
                        &group.id,
                        &sccache_runtime_env,
                        &group_compiler_env,
                    )?,
                    current_dir: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let outputs = ProcessRunner::new(workspace).run_parallel(specs)?;
        for (group, completed) in ready.into_iter().zip(outputs) {
            let log_path = report_root.join(format!("{}.log", group.id));
            let mut log = completed.output.stdout;
            log.extend_from_slice(&completed.output.stderr);
            bijux_dna_infra::write_bytes(&log_path, &log)?;
            let exit_code = completed.output.status.code().unwrap_or(1);
            let duration_milliseconds = duration_milliseconds(completed.duration);
            let group_lock_wait_events = cargo_lock_wait_events(&log);
            compiler_lock_wait_events += group_lock_wait_events;
            if group_lock_wait_events > 0 {
                compiler_lock_wait_duration_upper_bound_milliseconds += duration_milliseconds;
                compiler_lock_wait_targets.insert(group.id.clone());
            }
            group_durations.insert(group.id.clone(), duration_milliseconds);
            pending.remove(group.id.as_str());
            if exit_code == 0 {
                passed.insert(group.id.as_str());
                stdout.push_str(&format!(
                    "ci-fast group {}: passed in {} ms ({})\n",
                    group.id,
                    duration_milliseconds,
                    workspace.rel(&log_path).display()
                ));
            } else {
                failed.insert(group.id.as_str());
                stderr.push_str(&format!(
                    "ci-fast group {}: failed with exit {} ({})\n",
                    group.id,
                    exit_code,
                    workspace.rel(&log_path).display()
                ));
            }
            receipts.push(serde_json::json!({
                "id": group.id,
                "status": if exit_code == 0 { "passed" } else { "failed" },
                "dependencies": group.dependencies,
                "command": std::iter::once(group.program.as_str())
                    .chain(group.args.iter().map(String::as_str))
                    .collect::<Vec<_>>(),
                "duration_milliseconds": duration_milliseconds,
                "compiler_lock_wait_events": group_lock_wait_events,
                "build_target": build_authority.target.join(&group.id),
                "exit_code": exit_code,
                "log": workspace.rel(&log_path).to_string_lossy(),
            }));
        }
    }

    let wall_time_milliseconds = duration_milliseconds(started.elapsed());
    let all_passed = failed.is_empty() && passed.len() == contract.groups.len();
    let source_commit = source_commit(workspace)?;
    let source_worktree_clean = source_worktree_clean(workspace)?;
    let requested_cache_state = std::env::var("BIJUX_CI_CACHE_STATE")
        .unwrap_or_else(|_| "unspecified".to_string())
        .to_ascii_lowercase();
    let actual_os = std::env::consts::OS.to_string();
    let actual_architecture = std::env::consts::ARCH.to_string();
    let actual_rust_toolchain = rust_toolchain_version(workspace)?;
    let actual_compiler_wrapper = compiler_wrapper_identity(workspace)?;
    let cache_after = compiler_wrapper.as_deref().and_then(|wrapper| {
        read_sccache_stats(workspace, wrapper, &sccache_runtime_env).ok().flatten()
    });
    let cache_delta = cache_stats_delta(cache_before.as_ref(), cache_after.as_ref());
    let shared_cache_authority =
        cache_delta.as_ref().is_some_and(|delta| !delta.cache_location.contains(&source_commit));
    let shared_build_authority = build_authority.shared
        && !build_authority.target.to_string_lossy().contains(&source_commit);
    let cache_reuse_mode = verified_warm_cache_mode(
        &requested_cache_state,
        shared_cache_authority,
        shared_build_authority,
        cache_delta.as_ref(),
        contract.minimum_cache_hit_ratio_basis_points,
    );
    let observed_warm_cache = cache_reuse_mode.is_some();
    let cache_state = if observed_warm_cache { "warm_verified" } else { "unverified" };
    let eligible_reference_observation = all_passed
        && source_worktree_clean
        && observed_warm_cache
        && actual_os == contract.reference_runner.os
        && actual_architecture == contract.reference_runner.architecture
        && actual_rust_toolchain == contract.reference_runner.rust_toolchain
        && actual_compiler_wrapper.as_deref()
            == Some(contract.reference_runner.compiler_wrapper.as_str());
    let history_path = ci_output_path(workspace, &contract.timing_history)?;
    let mut history = load_fast_timing_history(&history_path)?;
    if eligible_reference_observation {
        history.observations.push(FastTimingObservation {
            observed_at_utc: chrono::Utc::now().to_rfc3339(),
            source_commit: source_commit.clone(),
            os: actual_os.clone(),
            architecture: actual_architecture.clone(),
            rust_toolchain: actual_rust_toolchain.clone(),
            compiler_wrapper: actual_compiler_wrapper.clone().unwrap_or_default(),
            cache_state: cache_state.to_string(),
            cache_authority: cache_delta
                .as_ref()
                .map(|delta| delta.cache_location.clone())
                .unwrap_or_default(),
            build_authority: build_authority.target.display().to_string(),
            cache_hit_ratio_basis_points: cache_delta
                .as_ref()
                .map_or(0, |delta| delta.hit_ratio_basis_points),
            cache_reuse_mode: cache_reuse_mode.unwrap_or_default().to_string(),
            wall_time_milliseconds,
        });
        if history.observations.len() > contract.history_limit {
            let remove_count = history.observations.len() - contract.history_limit;
            history.observations.drain(..remove_count);
        }
    }
    write_json_pretty(&history_path, &serde_json::to_value(&history)?)?;
    let samples = reference_timing_samples(&history, &contract.reference_runner);
    let p50 = percentile(&samples, 50);
    let p95 = percentile(&samples, 95);
    let p95_within_target = if samples.len() >= 5 {
        p95.map(|value| value <= contract.warm_cache_target_milliseconds)
    } else {
        None
    };
    let critical_path = ci_fast_critical_path(&contract.groups, &group_durations).map(
        |(groups, duration_milliseconds)| {
            serde_json::json!({
                "groups": groups,
                "duration_milliseconds": duration_milliseconds,
            })
        },
    );
    let observed_at_utc = chrono::Utc::now().to_rfc3339();
    let contract_sha256 = bijux_dna_infra::hash_file_sha256(&contract_path)?;
    let dependency_graph = contract
        .groups
        .iter()
        .map(|group| {
            serde_json::json!({
                "id": group.id,
                "dependencies": group.dependencies,
                "command": std::iter::once(group.program.as_str())
                    .chain(group.args.iter().map(String::as_str))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    let compiler_lock_waits = serde_json::json!({
        "measurement": "cargo diagnostic event count with conservative affected-target wall-time upper bound",
        "event_count": compiler_lock_wait_events,
        "affected_targets": compiler_lock_wait_targets,
        "duration_upper_bound_milliseconds": compiler_lock_wait_duration_upper_bound_milliseconds,
    });
    let skipped_test_lanes = serde_json::to_value(&contract.excluded_lanes)?;
    let runner = serde_json::json!({
        "os": actual_os,
        "architecture": actual_architecture,
        "rust_toolchain": actual_rust_toolchain,
        "compiler_wrapper": actual_compiler_wrapper,
        "sccache_server_uds": sccache_runtime_env.iter().find_map(|(key, value)|
            (key == "SCCACHE_SERVER_UDS").then_some(value)),
        "sccache_runtime_tmpdir": sccache_runtime_env.iter().find_map(|(key, value)|
            (key == "TMPDIR").then_some(value)),
        "sccache_rustc_wrapper": group_compiler_env.iter().find_map(|(key, value)|
            (key == "RUSTC_WRAPPER").then_some(value)),
        "cache_state": cache_state,
        "cache_reuse_mode": cache_reuse_mode,
        "requested_cache_state": requested_cache_state,
        "shared_cache_authority": shared_cache_authority,
        "build_authority": build_authority.target,
        "build_target_isolation": "per_group_under_shared_authority",
        "shared_build_authority": shared_build_authority,
        "cache_before": cache_before,
        "cache_after": cache_after,
        "cache_delta": cache_delta,
        "source_worktree_clean": source_worktree_clean,
    });
    write_json_pretty(
        &ci_output_path(workspace, &contract.timing_report)?,
        &serde_json::json!({
            "schema_version": FAST_TIMING_SCHEMA_V5,
            "source_commit": source_commit.clone(),
            "contract_sha256": contract_sha256.clone(),
            "observed_at_utc": observed_at_utc.clone(),
            "runner": runner.clone(),
            "reference_runner": {
                "os": contract.reference_runner.os,
                "architecture": contract.reference_runner.architecture,
                "rust_toolchain": contract.reference_runner.rust_toolchain,
                "compiler_wrapper": contract.reference_runner.compiler_wrapper,
            },
            "warm_cache_target_milliseconds": contract.warm_cache_target_milliseconds,
            "minimum_cache_hit_ratio_basis_points": contract.minimum_cache_hit_ratio_basis_points,
            "eligible_reference_observation": eligible_reference_observation,
            "wall_time_milliseconds": wall_time_milliseconds,
            "all_passed": all_passed,
            "critical_path": critical_path.clone(),
            "dependency_graph": dependency_graph.clone(),
            "compiler_lock_waits": compiler_lock_waits.clone(),
            "skipped_test_lanes": skipped_test_lanes.clone(),
            "groups": receipts.clone(),
            "warm_reference_history": {
                "sample_count": samples.len(),
                "p50_milliseconds": p50,
                "p95_milliseconds": p95,
                "p95_within_target": p95_within_target,
                "minimum_claim_sample_count": 5,
            },
        }),
    )?;
    write_json_pretty(
        &ci_output_path(workspace, &contract.baseline_report)?,
        &serde_json::json!({
            "schema_version": CI_CRITICAL_PATH_BASELINE_SCHEMA_V1,
            "source_commit": source_commit,
            "contract_sha256": contract_sha256,
            "observed_at_utc": observed_at_utc,
            "runner": runner,
            "wall_time_milliseconds": wall_time_milliseconds,
            "all_passed": all_passed,
            "critical_path": critical_path,
            "dependency_graph": dependency_graph,
            "compiler_lock_waits": compiler_lock_waits,
            "skipped_test_lanes": skipped_test_lanes,
            "per_target_timings": receipts,
        }),
    )?;
    Ok(OpsCommandOutcome { exit_code: i32::from(!all_passed), stdout, stderr })
}

fn cargo_lock_wait_events(log: &[u8]) -> usize {
    String::from_utf8_lossy(log)
        .to_ascii_lowercase()
        .matches("blocking waiting for file lock")
        .count()
}

fn ci_fast_critical_path(
    groups: &[FastGroup],
    durations: &BTreeMap<String, u64>,
) -> Option<(Vec<String>, u64)> {
    fn visit(
        id: &str,
        groups: &BTreeMap<&str, &FastGroup>,
        durations: &BTreeMap<String, u64>,
        memo: &mut BTreeMap<String, (Vec<String>, u64)>,
    ) -> (Vec<String>, u64) {
        if let Some(cached) = memo.get(id) {
            return cached.clone();
        }
        let group = groups[id];
        let mut path = group
            .dependencies
            .iter()
            .map(|dependency| visit(dependency, groups, durations, memo))
            .max_by_key(|(_, duration)| *duration)
            .unwrap_or_default();
        path.0.push(id.to_string());
        path.1 += durations.get(id).copied().unwrap_or_default();
        memo.insert(id.to_string(), path.clone());
        path
    }

    let groups_by_id =
        groups.iter().map(|group| (group.id.as_str(), group)).collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::new();
    groups
        .iter()
        .map(|group| visit(&group.id, &groups_by_id, durations, &mut memo))
        .max_by_key(|(_, duration)| *duration)
}

fn ci_fast_group_env(
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

fn materialize_ci_fast_compiler_wrapper(
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

fn resolve_ci_fast_sccache_runtime_env(
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

fn start_owned_sccache_server<'a>(
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

fn verify_sccache_compiler(
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

fn ci_fast_build_authority(workspace: &Workspace) -> Result<FastBuildAuthority> {
    resolve_ci_fast_build_authority_with_scratch_roots(
        workspace,
        std::env::var_os("BIJUX_CI_FAST_TARGET_DIR").map(PathBuf::from),
        &scheduler_scratch_roots(),
    )
}

#[cfg(test)]
fn resolve_ci_fast_build_authority(
    workspace: &Workspace,
    configured: Option<PathBuf>,
) -> Result<FastBuildAuthority> {
    resolve_ci_fast_build_authority_with_scratch_roots(workspace, configured, &[])
}

fn resolve_ci_fast_build_authority_with_scratch_roots(
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

fn scheduler_scratch_roots() -> Vec<PathBuf> {
    ["SNIC_TMP", "SLURM_TMPDIR"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
        .collect()
}

pub(in super::super) fn tooling_ci_slow(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-slow", args)?;
    run_make_target(workspace, "_ci-slow")
}

pub(in super::super) fn tooling_ci_test(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-test", args)?;
    let mut stdout = String::new();
    run_check_ids(&mut stdout, &["check-artifact-env-contract", "check-ssot-guardrails"])?;
    set_assets_readonly(workspace, true)?;
    let envs = ci_test_env(workspace, false)?;
    let expr = resolved_nextest_expression(workspace, false)?;
    let nextest_config =
        env_or_default("NEXTEST_CONFIG", "--config-file configs/rust/nextest.toml");
    let test_features = env_or_default("TEST_FEATURES", "--all-features");
    let no_tests = env_or_default("NEXTEST_NO_TESTS", "pass");
    let mut nextest_args = std::iter::once("nextest".to_string())
        .chain(std::iter::once("run".to_string()))
        .chain(nextest_config.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--workspace".to_string()))
        .chain(test_features.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--profile".to_string()))
        .chain(std::iter::once(resolved_nextest_profile(false)?))
        .chain(std::iter::once("--test-threads".to_string()))
        .chain(std::iter::once(resolved_nextest_threads(false)?))
        .chain(std::iter::once("--no-tests".to_string()))
        .chain(std::iter::once(no_tests))
        .collect::<Vec<_>>();
    let run_ignored = resolved_run_ignored(false)?;
    if !run_ignored.is_empty() {
        nextest_args.extend(run_ignored.split_whitespace().map(ToOwned::to_owned));
    }
    if let Some(value) = expr {
        nextest_args.push("-E".to_string());
        nextest_args.push(value);
    }
    let outcome = run_program_with_env(workspace, "cargo", &nextest_args, &envs);
    let restore = set_assets_readonly(workspace, false);
    let mut combined = OpsCommandOutcome::success(stdout);
    let test_outcome = outcome?;
    combined = merge_outcomes(combined, test_outcome);
    restore?;
    run_check_ids(&mut combined.stdout, &["check-artifact-env-contract"])?;
    Ok(combined)
}

pub(in super::super) fn tooling_ci_test_all_sharded(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        ensure_help_only("ci-test-all-sharded", args)?;
    }
    let contract_path = workspace.path("configs/ci/full-shards.json");
    let contract: FullShardContract = serde_json::from_slice(&fs::read(&contract_path)?)
        .with_context(|| format!("parse {}", contract_path.display()))?;
    validate_full_shard_contract(&contract)?;
    let preflight_only = matches!(args, [single] if single == "--preflight-only");
    let selected_shards = select_full_shards(&contract, if preflight_only { &[] } else { args })?;
    validate_full_shard_runtime_dependency_catalog(workspace, &contract.runtime_dependencies)?;

    let storage = resolve_full_shard_storage_roots(
        workspace,
        &contract,
        std::env::var_os(FULL_SHARD_SCRATCH_ROOT_ENV).map(PathBuf::from),
    )?;
    let report_root = storage.retained_report_root.clone();
    let build_authority_path = workspace.path(&contract.build_authority);
    let build_authority = read_ci_build_authority(&build_authority_path)?;
    require_ci_test_executor(workspace, &build_authority)?;
    let build_authority_sha256 = bijux_dna_infra::hash_file_sha256(&build_authority_path)?;
    bijux_dna_infra::ensure_dir(&report_root)?;
    let runtime_dependencies = observe_full_shard_runtime_dependencies(
        workspace,
        &contract.runtime_dependencies,
        &ci_output_path(workspace, &contract.runtime_dependency_report)?,
    )?;
    if preflight_only {
        return Ok(OpsCommandOutcome::success(format!(
            "full workspace runtime preflight passed; report: {}\n",
            contract.runtime_dependency_report
        )));
    }
    let started = Instant::now();
    let source_snapshot = TrackedSourceSnapshot::capture(workspace)?;
    let source_identity = source_snapshot.commit();
    let source_patch_sha256 = source_snapshot.patch_sha256();
    let source_clean = source_worktree_clean(workspace)?;
    write_json_pretty(
        &ci_output_path(workspace, "artifacts/ci/full-gate-timing.json")?,
        &serde_json::json!({
            "schema_version": "bijux.ci.full_gate_timing.v4",
            "state": "running",
            "source_commit": source_identity,
            "source_worktree_clean": source_clean,
            "source_patch_sha256": source_patch_sha256,
            "all_passed": false,
        }),
    )?;
    let contract_sha256 = bijux_dna_infra::hash_file_sha256(&contract_path)?;
    let archive_roots = full_archive_roots(&storage, &contract)?;
    validate_full_archive_roots(&storage, &archive_roots)?;
    let shard_roots = selected_shards
        .iter()
        .map(|shard| full_shard_roots(&storage, &contract, shard))
        .collect::<Result<Vec<_>>>()?;
    validate_full_shard_root_set(&storage, &shard_roots)?;
    for roots in &shard_roots {
        anyhow::ensure!(
            !paths_overlap(&roots.artifact_root, &archive_roots.cargo_target),
            "shard cleanup overlaps compilation products"
        );
    }
    let compilation_cache = CompilationCache::acquire(&storage.scratch_root)?;
    anyhow::ensure!(
        compilation_cache.target() == archive_roots.cargo_target,
        "archive target differs from its exclusive cache owner"
    );
    let mut ephemera = FullShardEphemera::new(&storage.scratch_root, &archive_roots, &shard_roots);
    prepare_full_archive_roots(&archive_roots)?;
    let preparation_started = Instant::now();
    let archive_workspace = if source_snapshot.is_clean() {
        let path = storage.scratch_root.join("compile-cache/workspace");
        source_snapshot.materialize_cached_clean(&path)?;
        path
    } else {
        materialize_full_shard_workspace(&source_snapshot, &storage.scratch_root, "archive-build")?
    };
    let archive_preparation = preparation_started.elapsed();
    let archive_spec = ParallelProcessSpec {
        label: "archive-build".to_string(),
        program: "cargo".to_string(),
        args: full_archive_args(&archive_roots.archive_file),
        envs: full_archive_env(&archive_roots, contract.archive_build_jobs)?,
        current_dir: Some(archive_workspace.clone()),
    };
    let mut archive_outputs =
        ProcessRunner::new(workspace).run_parallel_logged(vec![archive_spec], &report_root)?;
    let archive_output = archive_outputs.pop().context("archive build was not collected")?;
    let mut archive_log = archive_output.output.stdout.clone();
    archive_log.extend_from_slice(&archive_output.output.stderr);
    bijux_dna_infra::write_bytes(&archive_roots.build_log, &archive_log)?;
    let archive_exit_code = archive_output.output.status.code().unwrap_or(1);
    let archive_passed = archive_exit_code == 0 && archive_roots.archive_file.is_file();
    let archive_sha256 = archive_passed
        .then(|| bijux_dna_infra::hash_file_sha256(&archive_roots.archive_file))
        .transpose()?;
    if archive_passed {
        set_full_archive_readonly(&archive_roots.archive_file)?;
    }
    let mut archive_receipt = serde_json::json!({
        "schema_version": "bijux.ci.full_archive_build.v1",
        "source_commit": source_identity,
        "source_worktree_clean": source_clean,
        "source_patch_sha256": source_patch_sha256,
        "contract_sha256": contract_sha256,
        "build_authority_sha256": build_authority_sha256,
        "workspace": workspace.rel(&archive_workspace).to_string_lossy(),
        "artifact": workspace.rel(&archive_roots.archive_file).to_string_lossy(),
        "artifact_sha256": archive_sha256,
        "artifact_present": archive_roots.archive_file.is_file(),
        "cargo_target_dir": archive_roots.cargo_target.display().to_string(),
        "build_jobs": contract.archive_build_jobs,
        "duration_milliseconds": archive_output.duration.as_millis(),
        "exit_code": archive_exit_code,
        "log": workspace.rel(&archive_roots.build_log).to_string_lossy(),
        "stdout_log": workspace.rel(&report_root.join("archive-build.stdout.log")).to_string_lossy(),
        "stderr_log": workspace.rel(&report_root.join("archive-build.stderr.log")).to_string_lossy(),
        "read_only_during_shard_execution": archive_passed,
    });
    // Retain completed build evidence even when discovery or execution is interrupted.
    write_json_pretty(&report_root.join("archive-build.receipt.json"), &archive_receipt)?;
    let mut stdout = String::from_utf8_lossy(&archive_output.output.stdout).into_owned();
    let mut stderr = String::from_utf8_lossy(&archive_output.output.stderr).into_owned();
    if !archive_passed {
        finish_full_shard_receipt(
            &ci_output_path(workspace, "artifacts/ci/full-gate-timing.json")?,
            serde_json::json!({
                "schema_version": "bijux.ci.full_gate_timing.v4",
            "state": "complete",
            "source_commit": source_identity,
            "source_worktree_clean": source_clean,
            "source_patch_sha256": source_patch_sha256,
                "contract_sha256": contract_sha256,
                "build_authority": workspace.rel(&build_authority_path).to_string_lossy(),
                "build_authority_sha256": build_authority_sha256,
                "runtime_dependencies": runtime_dependencies,
                "archive": archive_receipt,
                "evidence_retention": {
                    "policy": "bounded_logs_and_receipts",
                    "archive_retained": false,
                    "build_targets_retained": true,
                    "compilation_cache_target": compilation_cache.target(),
                    "compilation_workspace": archive_workspace,
                    "compilation_workspace_retained": source_snapshot.is_clean(),
                    "isolated_workspaces_retained": false,
                    "shard_runtime_artifacts_retained": false,
                },
                "storage": full_shard_storage_receipt(&storage),
                "execution_scope": full_shard_execution_scope(&contract, &selected_shards, false),
                "wall_time_milliseconds": started.elapsed().as_millis(),
                "all_passed": false,
                "shards": [],
            }),
            started,
            &mut ephemera,
        )?;
        return Ok(OpsCommandOutcome { exit_code: 1, stdout, stderr });
    }
    prepare_full_archive_extraction(&archive_roots.archive_extract)?;
    let inventory_spec = ParallelProcessSpec {
        label: "test-inventory".to_string(),
        program: "cargo".to_string(),
        args: full_inventory_args(&archive_roots),
        envs: full_archive_env(&archive_roots, contract.archive_build_jobs)?,
        current_dir: Some(archive_workspace.clone()),
    };
    let mut inventory_outputs =
        ProcessRunner::new(workspace).run_parallel_logged(vec![inventory_spec], &report_root)?;
    let inventory_output = inventory_outputs.pop().context("test inventory was not collected")?;
    anyhow::ensure!(
        inventory_output.output.status.success(),
        "full test inventory failed; see test-inventory.stderr.log"
    );
    let expected_inventory = ExpectedTestInventory::parse(&inventory_output.output.stdout)?;
    let executables = CompiledWorkspaceExecutables::parse(&inventory_output.output.stdout)?;
    let inventory_log = report_root.join("test-inventory.stdout.log");
    let inventory_receipt = serde_json::json!({
        "schema_version": "bijux.ci.full_test_inventory.v1",
        "path": workspace.rel(&inventory_log).to_string_lossy(),
        "sha256": bijux_dna_infra::hash_file_sha256(&inventory_log)?,
        "workspace_executables": &executables,
        "expected_case_count": expected_inventory.case_count(),
        "expected_ignored_case_count": expected_inventory.ignored_case_count(),
        "duration_milliseconds": inventory_output.duration.as_millis(),
        "source_commit": source_identity,
        "source_patch_sha256": source_patch_sha256,
        "archive_sha256": archive_sha256,
    });
    write_json_pretty(&report_root.join("test-inventory.receipt.json"), &inventory_receipt)?;
    seal_full_archive_extraction(&archive_roots.archive_extract)?;
    let shard_plans = selected_shards
        .iter()
        .map(|shard| {
            let plan = ShardDiscoveryPlan::from_inventory(
                &expected_inventory,
                &shard.partition,
                &build_authority.test_executor.version,
            )?;
            anyhow::ensure!(
                plan.expected_case_count > 0,
                "selected shard {} has no expected cases",
                shard.id
            );
            Ok(plan)
        })
        .collect::<Result<Vec<_>>>()?;
    let preparation_started = Instant::now();
    let shard_workspaces = selected_shards
        .iter()
        .map(|shard| {
            materialize_full_shard_workspace(&source_snapshot, &storage.scratch_root, &shard.id)
        })
        .collect::<Result<Vec<_>>>()?;
    let shard_preparation = preparation_started.elapsed();
    let specs = selected_shards
        .iter()
        .zip(&shard_workspaces)
        .zip(&shard_roots)
        .zip(&shard_plans)
        .map(|(((shard, shard_workspace), roots), plan)| {
            let mut args = full_shard_args(shard, &archive_roots.archive_extract, shard_workspace)?;
            args.extend(["--filterset".to_string(), plan.filterset.clone()]);
            let mut envs = full_shard_env(roots, shard_workspace)?;
            envs.extend(
                executables.remapped_environment(&archive_roots.archive_extract.join("target"))?,
            );
            Ok(ParallelProcessSpec {
                label: shard.id.clone(),
                program: "cargo".to_string(),
                args,
                envs,
                current_dir: Some(shard_workspace.clone()),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let outputs = ProcessRunner::new(workspace).run_parallel_logged(specs, &report_root)?;
    let coverage = CoverageAttestation::reconcile_planned(
        &expected_inventory,
        outputs.iter().zip(&shard_plans).map(|(completed, plan)| {
            (completed.label.as_str(), completed.output.stdout.as_slice(), plan)
        }),
    );
    let mut all_passed = coverage.selected_shards_valid
        && (selected_shards.len() != contract.shards.len() || coverage.complete);
    let mut shard_receipts = Vec::with_capacity(outputs.len());
    for (((shard, shard_workspace), roots), completed) in
        selected_shards.iter().zip(&shard_workspaces).zip(&shard_roots).zip(outputs)
    {
        let log_path = report_root.join(format!("{}.log", shard.id));
        let mut log = completed.output.stdout.clone();
        log.extend_from_slice(&completed.output.stderr);
        bijux_dna_infra::write_bytes(&log_path, &log)?;
        let exit_code = completed.output.status.code().unwrap_or(1);
        all_passed &= exit_code == 0;
        stdout.push_str(&String::from_utf8_lossy(&completed.output.stdout));
        stderr.push_str(&String::from_utf8_lossy(&completed.output.stderr));
        shard_receipts.push(serde_json::json!({
            "id": completed.label,
            "partition": shard.partition,
            "workspace": workspace.rel(shard_workspace).to_string_lossy(),
            "environment_roots": full_shard_root_receipt(roots, &archive_roots, shard_workspace),
            "duration_milliseconds": completed.duration.as_millis(),
            "exit_code": exit_code,
            "log": workspace.rel(&log_path).to_string_lossy(),
            "stdout_log": workspace.rel(&report_root.join(format!("{}.stdout.log", shard.id))).to_string_lossy(),
            "stderr_log": workspace.rel(&report_root.join(format!("{}.stderr.log", shard.id))).to_string_lossy(),
        }));
    }
    let archive_sha256_after = bijux_dna_infra::hash_file_sha256(&archive_roots.archive_file)?;
    let archive_checksum_stable = archive_sha256.as_deref() == Some(archive_sha256_after.as_str());
    all_passed &= archive_checksum_stable;
    archive_receipt["artifact_sha256_after_execution"] =
        serde_json::Value::String(archive_sha256_after);
    archive_receipt["checksum_stable"] = serde_json::Value::Bool(archive_checksum_stable);
    finish_full_shard_receipt(
        &ci_output_path(workspace, "artifacts/ci/full-gate-timing.json")?,
        serde_json::json!({
            "schema_version": "bijux.ci.full_gate_timing.v4",
            "state": "complete",
            "source_commit": source_identity,
            "source_worktree_clean": source_clean,
            "source_patch_sha256": source_patch_sha256,
            "contract_sha256": contract_sha256,
            "build_authority": workspace.rel(&build_authority_path).to_string_lossy(),
            "build_authority_sha256": build_authority_sha256,
            "runtime_dependencies": runtime_dependencies,
            "archive": archive_receipt,
            "evidence_retention": {
                "policy": "bounded_logs_and_receipts",
                "archive_retained": false,
                "build_targets_retained": true,
                    "compilation_cache_target": compilation_cache.target(),
                    "compilation_workspace": archive_workspace,
                    "compilation_workspace_retained": source_snapshot.is_clean(),
                "isolated_workspaces_retained": false,
                "shard_runtime_artifacts_retained": false,
            },
            "storage": full_shard_storage_receipt(&storage),
            "execution_scope": full_shard_execution_scope(&contract, &selected_shards, coverage.complete),
            "source_preparation": {
                "archive_milliseconds": archive_preparation.as_millis(),
                "shards_milliseconds": shard_preparation.as_millis(),
            },
            "discovery_plans": selected_shards.iter().zip(&shard_plans).map(|(shard, plan)| {
                serde_json::json!({"shard_id": shard.id, "plan": plan})
            }).collect::<Vec<_>>(),
            "expected_inventory": inventory_receipt,
            "coverage": {
                "observed_test_execution": coverage,
                "workspace": true,
                "all_targets": true,
                "all_features": true,
                "run_ignored": "all",
                "overlap_policy": "exactly_once",
                "workspace_isolation": contract.workspace_isolation,
                "output_isolation": "shared_read_only_extraction_with_disjoint_workspaces_and_runtime_roots"
            },
            "wall_time_milliseconds": started.elapsed().as_millis(),
            "all_passed": all_passed,
            "shards": shard_receipts,
        }),
        started,
        &mut ephemera,
    )?;
    Ok(OpsCommandOutcome { exit_code: i32::from(!all_passed), stdout, stderr })
}

/// Publish a terminal result only after the owned cleanup attempt has completed.
fn finish_full_shard_receipt(
    path: &Path,
    mut receipt: serde_json::Value,
    started: Instant,
    ephemera: &mut FullShardEphemera,
) -> Result<()> {
    let cleanup_started = Instant::now();
    let cleanup = ephemera.retain_bounded_evidence();
    // A terminal receipt must describe the last attempt, without a hidden Drop retry.
    ephemera.armed = false;
    receipt["cleanup"] = serde_json::json!({
        "completed": cleanup.is_ok(),
        "duration_milliseconds": cleanup_started.elapsed().as_millis(),
        "error": cleanup.as_ref().err().map(|error| format!("{error:#}")),
    });
    receipt["evidence_retention"]["archive_retained"] =
        serde_json::json!(ephemera.archive_root.symlink_metadata().is_ok());
    let compilation_target = receipt["evidence_retention"]["compilation_cache_target"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| ephemera.archive_root.join("build"));
    receipt["evidence_retention"]["build_targets_retained"] =
        serde_json::json!(compilation_target.exists());
    receipt["evidence_retention"]["isolated_workspaces_retained"] =
        serde_json::json!(ephemera.scratch_workspaces.symlink_metadata().is_ok());
    receipt["evidence_retention"]["shard_runtime_artifacts_retained"] = serde_json::json!(ephemera
        .shard_artifact_roots
        .iter()
        .any(|root| root.symlink_metadata().is_ok()));
    receipt["wall_time_milliseconds"] = serde_json::json!(started.elapsed().as_millis());
    receipt["all_passed"] =
        serde_json::json!(receipt["all_passed"].as_bool() == Some(true) && cleanup.is_ok());
    write_json_pretty(path, &receipt)?;
    cleanup
}

fn select_full_shards(contract: &FullShardContract, args: &[String]) -> Result<Vec<FullShard>> {
    if args.is_empty() {
        return Ok(contract.shards.clone());
    }
    if !args.len().is_multiple_of(2) {
        anyhow::bail!("ci-test-all-sharded expects repeated --shard <id> pairs");
    }
    let mut selected_ids = BTreeSet::new();
    let mut selected = Vec::with_capacity(args.len() / 2);
    for pair in args.chunks_exact(2) {
        if pair[0] != "--shard" || pair[1].trim().is_empty() {
            anyhow::bail!("ci-test-all-sharded expects repeated --shard <id> pairs");
        }
        if !selected_ids.insert(pair[1].as_str()) {
            anyhow::bail!("full shard selection repeats {}", pair[1]);
        }
        selected.push(
            contract
                .shards
                .iter()
                .find(|shard| shard.id == pair[1])
                .cloned()
                .with_context(|| format!("unknown full shard selection {}", pair[1]))?,
        );
    }
    Ok(selected)
}

fn full_shard_execution_scope(
    contract: &FullShardContract,
    selected_shards: &[FullShard],
    observed_complete: bool,
) -> serde_json::Value {
    serde_json::json!({
        "mode": if selected_shards.len() == contract.shards.len() {
            "complete"
        } else {
            "affected_recovery"
        },
        "all_declared_shards_selected": selected_shards.len() == contract.shards.len(),
        "complete_contract_coverage": selected_shards.len() == contract.shards.len() && observed_complete,
        "selected_shard_ids": selected_shards.iter().map(|shard| &shard.id).collect::<Vec<_>>(),
        "selected_partitions": selected_shards
            .iter()
            .map(|shard| &shard.partition)
            .collect::<Vec<_>>(),
        "contract_shard_count": contract.shards.len(),
        "selected_shard_count": selected_shards.len(),
    })
}

fn remove_full_shard_ephemera(path: &Path) -> Result<()> {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect full shard ephemeral path {}", path.display()))
        }
    };
    if metadata.file_type().is_symlink() {
        anyhow::bail!("full shard ephemeral path is a symlink: {}", path.display());
    }
    make_full_shard_ephemera_removable(path)?;
    if metadata.is_dir() {
        std::fs::remove_dir_all(path)
            .with_context(|| format!("remove full shard ephemeral directory {}", path.display()))
    } else {
        std::fs::remove_file(path)
            .with_context(|| format!("remove full shard ephemeral file {}", path.display()))
    }
}

fn make_full_shard_ephemera_removable(path: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = entry.with_context(|| {
            format!("inspect full shard ephemeral path below {}", path.display())
        })?;
        let metadata = entry.path().symlink_metadata().with_context(|| {
            format!("read full shard ephemeral metadata {}", entry.path().display())
        })?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let mut permissions = metadata.permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let owner_access = if metadata.is_dir() { 0o700 } else { 0o600 };
            permissions.set_mode(permissions.mode() | owner_access);
        }
        #[cfg(not(unix))]
        permissions.set_readonly(false);
        std::fs::set_permissions(entry.path(), permissions).with_context(|| {
            format!("restore owner access to full shard ephemera {}", entry.path().display())
        })?;
    }
    Ok(())
}

fn read_ci_build_authority(path: &Path) -> Result<CiBuildAuthority> {
    let authority: CiBuildAuthority = serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    validate_ci_build_authority(&authority)?;
    Ok(authority)
}

fn require_ci_test_executor(workspace: &Workspace, authority: &CiBuildAuthority) -> Result<()> {
    let output = ProcessRunner::new(workspace).run_owned(
        "cargo",
        &[authority.test_executor.subcommand.clone(), "--version".to_string()],
    )?;
    let observed = if output.status.success() {
        String::from_utf8(output.stdout)?.trim().to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    };
    validate_ci_test_executor_observation(authority, output.status.success(), &observed)
}

fn observe_full_shard_runtime_dependencies(
    workspace: &Workspace,
    dependencies: &[FullShardRuntimeDependency],
    report_path: &Path,
) -> Result<Vec<serde_json::Value>> {
    observe_full_shard_runtime_dependencies_with_env(workspace, dependencies, report_path, None)
}

fn observe_full_shard_runtime_dependencies_with_env(
    workspace: &Workspace,
    dependencies: &[FullShardRuntimeDependency],
    report_path: &Path,
    execution_env: Option<&BTreeMap<String, String>>,
) -> Result<Vec<serde_json::Value>> {
    let mut observations = Vec::with_capacity(dependencies.len());
    let mut all_available = true;
    for dependency in dependencies {
        let (available, observed, error) =
            match bijux_dna_runner::api::run_command_with_context_and_timeout(
                &dependency.executable,
                &dependency.version_args,
                Some(&workspace.root),
                execution_env,
                Duration::from_secs(30),
            ) {
                Ok(timed) => {
                    let output = timed.output;
                    let rendered = if output.stdout.is_empty() {
                        output.stderr.trim().to_string()
                    } else {
                        output.stdout.trim().to_string()
                    };
                    let available = !timed.timed_out
                        && output.exit_code == 0
                        && rendered.lines().next()
                            == Some(dependency.expected_version_line.as_str());
                    (available, rendered, None)
                }
                Err(error) => (false, String::new(), Some(format!("{error:#}"))),
            };
        all_available &= available;
        observations.push(serde_json::json!({
            "id": dependency.id,
            "executable": dependency.executable,
            "version_args": dependency.version_args,
            "expected_version_line": dependency.expected_version_line,
            "provisioning": dependency.provisioning,
            "available": available,
            "observed": observed,
            "error": error,
        }));
    }
    write_json_pretty(
        report_path,
        &serde_json::json!({
            "schema_version": "bijux.ci.full_shard_runtime_dependencies.v1",
            "all_available": all_available,
            "dependencies": &observations,
        }),
    )?;
    if !all_available {
        anyhow::bail!(
            "full workspace shard runtime dependencies are unavailable; see {}",
            report_path.display()
        );
    }
    Ok(observations)
}

fn validate_ci_test_executor_observation(
    authority: &CiBuildAuthority,
    available: bool,
    observed: &str,
) -> Result<()> {
    if !available {
        anyhow::bail!(
            "required CI test executor {} {} is unavailable ({observed}); run `cargo run -p bijux-dna-dev -- tooling run ci-install-tools` once before launching shards",
            authority.test_executor.package,
            authority.test_executor.version
        );
    }
    let expected =
        format!("{} {}", authority.test_executor.package, authority.test_executor.version);
    if !observed.lines().next().is_some_and(|line| line.starts_with(&expected)) {
        anyhow::bail!(
            "CI test executor version mismatch: expected {expected}, observed {observed}"
        );
    }
    Ok(())
}

fn full_archive_args(archive_file: &Path) -> Vec<String> {
    vec![
        "nextest".to_string(),
        "archive".to_string(),
        "--workspace".to_string(),
        "--all-targets".to_string(),
        "--all-features".to_string(),
        "--locked".to_string(),
        "--config-file".to_string(),
        "configs/rust/nextest.toml".to_string(),
        "--profile".to_string(),
        "full".to_string(),
        "--archive-file".to_string(),
        archive_file.display().to_string(),
    ]
}

fn full_inventory_args(archive: &FullArchiveRoots) -> Vec<String> {
    vec![
        "nextest".to_string(),
        "list".to_string(),
        "--archive-file".to_string(),
        archive.archive_file.display().to_string(),
        "--extract-to".to_string(),
        archive.archive_extract.display().to_string(),
        "--config-file".to_string(),
        "configs/rust/nextest.toml".to_string(),
        "--profile".to_string(),
        "full".to_string(),
        "--message-format".to_string(),
        "json".to_string(),
    ]
}

fn full_shard_args(
    shard: &FullShard,
    archive_extract: &Path,
    workspace_remap: &Path,
) -> Result<Vec<String>> {
    let mut args = vec![
        "nextest".to_string(),
        "run".to_string(),
        "--message-format".to_string(),
        "libtest-json-plus".to_string(),
        "--message-format-version".to_string(),
        "0.1".to_string(),
        "--no-tests".to_string(),
        "fail".to_string(),
        "--config-file".to_string(),
        "configs/rust/nextest.toml".to_string(),
        "--profile".to_string(),
        "full".to_string(),
        "--run-ignored".to_string(),
        "all".to_string(),
        "--retries".to_string(),
        "0".to_string(),
        "--partition".to_string(),
        shard.partition.clone(),
    ];
    let target = archive_extract.join("target");
    let reuse = NextestReuseInputs::new(
        target.join("nextest/cargo-metadata.json"),
        target.join("nextest/binaries-metadata.json"),
        target,
    )?;
    args.extend(reuse.arguments(workspace_remap)?);
    Ok(args)
}

fn materialize_full_shard_workspace(
    snapshot: &TrackedSourceSnapshot,
    report_root: &Path,
    identity: &str,
) -> Result<PathBuf> {
    anyhow::ensure!(
        Path::new(identity).components().count() == 1
            && matches!(Path::new(identity).components().next(), Some(Component::Normal(_))),
        "shard workspace identity must be one directory name"
    );
    let shard_workspace = report_root.join("workspaces").join(identity);
    if shard_workspace.symlink_metadata().is_ok() {
        std::fs::remove_dir_all(&shard_workspace).with_context(|| {
            format!("remove stale shard workspace {}", shard_workspace.display())
        })?;
    }
    let shard_parent = shard_workspace.parent().context("shard workspace has no parent")?;
    bijux_dna_infra::ensure_dir(shard_parent)?;
    snapshot.materialize(&shard_workspace)?;
    Ok(shard_workspace)
}

fn full_shard_roots(
    storage: &FullShardStorageRoots,
    contract: &FullShardContract,
    shard: &FullShard,
) -> Result<FullShardRoots> {
    let artifact_root = full_shard_scratch_path(
        &storage.scratch_root,
        &contract.report_root,
        &contract.shard_artifact_root,
    )?
    .join(&shard.id);
    Ok(FullShardRoots {
        cargo_target: isolated_build_tree(&artifact_root, "cargo"),
        test_target: isolated_build_tree(&artifact_root, "test"),
        coverage_target: isolated_build_tree(&artifact_root, "coverage"),
        test_tmp: artifact_root.join("tmp/test"),
        coverage_tmp: artifact_root.join("tmp/coverage"),
        test_profraw: artifact_root.join("coverage/profraw-test"),
        coverage_profraw: artifact_root.join("coverage/profraw-coverage"),
        artifact_root,
    })
}

fn full_archive_roots(
    storage: &FullShardStorageRoots,
    contract: &FullShardContract,
) -> Result<FullArchiveRoots> {
    let artifact_root = full_shard_scratch_path(
        &storage.scratch_root,
        &contract.report_root,
        &contract.archive_root,
    )?;
    Ok(FullArchiveRoots {
        archive_extract: artifact_root.join("extracted"),
        cargo_target: CompilationCache::target_under(&storage.scratch_root),
        archive_file: artifact_root.join("workspace-tests.tar.zst"),
        build_log: storage.retained_report_root.join("archive-build.log"),
        artifact_root,
    })
}

fn isolated_build_tree(artifact_root: &Path, consumer: &str) -> PathBuf {
    artifact_root.join("build").join(consumer)
}

fn ci_output_path(workspace: &Workspace, configured: &str) -> Result<PathBuf> {
    ci_output_path_under(&artifact_root_path(workspace)?, configured)
}

fn ci_output_path_under(artifact_root: &Path, configured: &str) -> Result<PathBuf> {
    validate_artifact_relative_path(configured)?;
    let suffix = Path::new(configured)
        .strip_prefix("artifacts")
        .context("full shard output path has no artifacts prefix")?;
    Ok(artifact_root.join(suffix))
}

fn resolve_full_shard_storage_roots(
    workspace: &Workspace,
    contract: &FullShardContract,
    configured_scratch_root: Option<PathBuf>,
) -> Result<FullShardStorageRoots> {
    let retained_report_root = ci_output_path(workspace, &contract.report_root)?;
    let scratch_root = configured_scratch_root.map_or_else(
        || artifact_root_path(workspace).map(|root| root.join("ci/full-shards-scratch")),
        Ok,
    )?;
    validate_absolute_normalized_path(&scratch_root, FULL_SHARD_SCRATCH_ROOT_ENV)?;
    if paths_overlap(&scratch_root, &retained_report_root) {
        anyhow::bail!(
            "full shard scratch root and retained report root overlap: {} and {}",
            scratch_root.display(),
            retained_report_root.display()
        );
    }
    Ok(FullShardStorageRoots { retained_report_root, scratch_root })
}

fn full_shard_scratch_path(
    scratch_root: &Path,
    configured_report_root: &str,
    configured_path: &str,
) -> Result<PathBuf> {
    let suffix = Path::new(configured_path)
        .strip_prefix(configured_report_root)
        .context("full shard scratch path is outside its declared report root")?;
    if suffix.as_os_str().is_empty() {
        anyhow::bail!("full shard scratch path must be a child of its declared report root");
    }
    Ok(scratch_root.join(suffix))
}

fn validate_absolute_normalized_path(path: &Path, authority: &str) -> Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        anyhow::bail!("{authority} must be absolute and normalized");
    }
    Ok(())
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn validate_full_archive_roots(
    storage: &FullShardStorageRoots,
    roots: &FullArchiveRoots,
) -> Result<()> {
    if roots
        .scratch_all()
        .into_iter()
        .any(|root| !root.is_absolute() || !root.starts_with(&storage.scratch_root))
    {
        anyhow::bail!("full shard archive root is outside scratch authority");
    }
    if !roots.build_log.is_absolute() || !roots.build_log.starts_with(&storage.retained_report_root)
    {
        anyhow::bail!("full shard archive log is outside retained report authority");
    }
    if !roots.archive_file.starts_with(&roots.artifact_root)
        || roots.cargo_target != CompilationCache::target_under(&storage.scratch_root)
        || paths_overlap(&roots.cargo_target, &roots.artifact_root)
        || !roots.archive_extract.starts_with(&roots.artifact_root)
        || paths_overlap(&roots.archive_extract, &roots.cargo_target)
        || paths_overlap(&roots.archive_extract, &roots.archive_file)
    {
        anyhow::bail!("full shard archive paths escape the archive scratch root");
    }
    Ok(())
}

fn prepare_full_archive_roots(roots: &FullArchiveRoots) -> Result<()> {
    bijux_dna_infra::ensure_dir(&roots.artifact_root)?;
    bijux_dna_infra::ensure_dir(&roots.cargo_target)?;
    for stale in [&roots.archive_file, &roots.build_log] {
        if stale.exists() {
            std::fs::remove_file(stale)
                .with_context(|| format!("remove stale full shard artifact {}", stale.display()))?;
        }
    }
    Ok(())
}

fn set_full_archive_readonly(path: &Path) -> Result<()> {
    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("read full shard archive metadata {}", path.display()))?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)
        .with_context(|| format!("make full shard archive read-only {}", path.display()))
}

fn prepare_full_archive_extraction(path: &Path) -> Result<()> {
    remove_full_shard_ephemera(path)?;
    bijux_dna_infra::ensure_dir(path)?;
    Ok(())
}

fn seal_full_archive_extraction(path: &Path) -> Result<()> {
    let metadata = path
        .symlink_metadata()
        .with_context(|| format!("read extracted archive root {}", path.display()))?;
    anyhow::ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "full archive extraction must be an owned directory: {}",
        path.display()
    );
    for entry in walkdir::WalkDir::new(path).follow_links(false).contents_first(true) {
        let entry =
            entry.with_context(|| format!("inspect extracted archive {}", path.display()))?;
        if !entry.file_type().is_symlink() {
            set_full_archive_readonly(entry.path())?;
        }
    }
    Ok(())
}

fn full_archive_env(roots: &FullArchiveRoots, build_jobs: usize) -> Result<Vec<(String, String)>> {
    bijux_dna_infra::ensure_dir(&roots.cargo_target)?;
    Ok(vec![
        ("ARTIFACT_ROOT".to_string(), roots.artifact_root.display().to_string()),
        ("ISO_ROOT".to_string(), roots.artifact_root.display().to_string()),
        ("CARGO_TARGET_DIR".to_string(), roots.cargo_target.display().to_string()),
        ("CARGO_BUILD_JOBS".to_string(), build_jobs.to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ])
}

fn validate_full_shard_root_set(
    storage: &FullShardStorageRoots,
    shard_roots: &[FullShardRoots],
) -> Result<()> {
    for roots in shard_roots {
        for root in roots.all() {
            if !root.is_absolute() || !root.starts_with(&storage.scratch_root) {
                anyhow::bail!(
                    "full shard output root is outside scratch authority: {}",
                    root.display()
                );
            }
        }
        let mut distinct = roots.all().into_iter().collect::<BTreeSet<_>>();
        distinct.remove(roots.artifact_root.as_path());
        if distinct.len() != roots.all().len() - 1 {
            anyhow::bail!("full shard runtime roots are not distinct");
        }
    }
    for (index, left) in shard_roots.iter().enumerate() {
        for right in shard_roots.iter().skip(index + 1) {
            if left
                .all()
                .into_iter()
                .any(|left_root| right.all().into_iter().any(|right_root| left_root == right_root))
            {
                anyhow::bail!("full shard output roots overlap");
            }
        }
    }
    Ok(())
}

fn full_shard_env(roots: &FullShardRoots, shard_workspace: &Path) -> Result<Vec<(String, String)>> {
    for root in roots.all() {
        bijux_dna_infra::ensure_dir(root)?;
    }
    let workspace = Workspace { root: shard_workspace.to_path_buf() };
    let outputs = workspace.admit_artifact_path(&workspace.path("artifacts"))?;
    bijux_dna_infra::ensure_dir(&outputs)?;
    Ok(vec![
        ("NEXTEST_EXPERIMENTAL_LIBTEST_JSON".to_string(), "1".to_string()),
        ("ARTIFACT_ROOT".to_string(), outputs.display().to_string()),
        ("ISO_ROOT".to_string(), outputs.display().to_string()),
        ("CARGO_TARGET_DIR".to_string(), roots.cargo_target.display().to_string()),
        ("TEST_TARGET_DIR".to_string(), roots.test_target.display().to_string()),
        ("COV_TARGET_DIR".to_string(), roots.coverage_target.display().to_string()),
        ("TEST_TMP_DIR".to_string(), roots.test_tmp.display().to_string()),
        ("COV_TMP_DIR".to_string(), roots.coverage_tmp.display().to_string()),
        ("TEST_PROFRAW_DIR".to_string(), roots.test_profraw.display().to_string()),
        ("COV_PROFRAW_DIR".to_string(), roots.coverage_profraw.display().to_string()),
        ("BIJUX_REPO_ROOT".to_string(), shard_workspace.display().to_string()),
        (
            "BIJUX_BENCHMARK_ROOT".to_string(),
            shard_workspace.join("benchmarks").display().to_string(),
        ),
        // Archived snapshot assertions retain their compile-time source paths. Shards
        // therefore fail with a rendered diff but never write pending source files.
        ("INSTA_UPDATE".to_string(), "no".to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ])
}

fn full_shard_root_receipt(
    roots: &FullShardRoots,
    archive: &FullArchiveRoots,
    shard_workspace: &Path,
) -> serde_json::Value {
    serde_json::json!({
        "repository_root": shard_workspace.display().to_string(),
        "benchmark_root": shard_workspace.join("benchmarks").display().to_string(),
        "snapshot_source_authority": "archive_compile_time_metadata",
        "snapshot_update_policy": "no",
        "artifact_root": shard_workspace.join("artifacts").display().to_string(),
        "iso_root": shard_workspace.join("artifacts").display().to_string(),
        "executor_scratch_root": roots.artifact_root.display().to_string(),
        "archive_extract_dir": archive.archive_extract.display().to_string(),
        "archive_extraction_ownership": "shared_read_only",
        "cargo_target_dir": roots.cargo_target.display().to_string(),
        "test_target_dir": roots.test_target.display().to_string(),
        "coverage_target_dir": roots.coverage_target.display().to_string(),
        "test_tmp_dir": roots.test_tmp.display().to_string(),
        "coverage_tmp_dir": roots.coverage_tmp.display().to_string(),
        "test_profraw_dir": roots.test_profraw.display().to_string(),
        "coverage_profraw_dir": roots.coverage_profraw.display().to_string(),
    })
}

fn full_shard_storage_receipt(storage: &FullShardStorageRoots) -> serde_json::Value {
    serde_json::json!({
        "retained_report_root": storage.retained_report_root.display().to_string(),
        "scratch_root": storage.scratch_root.display().to_string(),
        "scratch_root_environment": FULL_SHARD_SCRATCH_ROOT_ENV,
        "scratch_retained": false,
    })
}

impl FullShardRoots {
    fn all(&self) -> [&Path; 8] {
        [
            &self.artifact_root,
            &self.cargo_target,
            &self.test_target,
            &self.coverage_target,
            &self.test_tmp,
            &self.coverage_tmp,
            &self.test_profraw,
            &self.coverage_profraw,
        ]
    }
}

impl FullArchiveRoots {
    fn scratch_all(&self) -> [&Path; 4] {
        [&self.artifact_root, &self.cargo_target, &self.archive_file, &self.archive_extract]
    }
}

fn validate_full_shard_contract(contract: &FullShardContract) -> Result<()> {
    if contract.schema_version != FULL_SHARDS_SCHEMA_V5
        || !contract.coverage.workspace
        || !contract.coverage.all_targets
        || !contract.coverage.all_features
        || contract.coverage.run_ignored != "all"
        || contract.coverage.overlap_policy != "exactly_once"
        || contract.partition_strategy != "hash"
        || contract.workspace_isolation != "tracked_source_clones_with_shared_immutable_archive"
        || contract.build_authority != "configs/ci/build-authority.json"
        || contract.runtime_dependencies.is_empty()
        || !(1..=16).contains(&contract.archive_build_jobs)
        || !(2..=16).contains(&contract.shards.len())
    {
        anyhow::bail!("full shard coverage contract is incomplete");
    }
    validate_artifact_relative_path(&contract.archive_root)?;
    validate_artifact_relative_path(&contract.shard_artifact_root)?;
    validate_artifact_relative_path(&contract.report_root)?;
    validate_artifact_relative_path(&contract.runtime_dependency_report)?;
    let report_root = Path::new(&contract.report_root);
    let archive_root = Path::new(&contract.archive_root);
    let shard_artifact_root = Path::new(&contract.shard_artifact_root);
    let runtime_dependency_report = Path::new(&contract.runtime_dependency_report);
    if archive_root == report_root
        || !archive_root.starts_with(report_root)
        || shard_artifact_root == report_root
        || !shard_artifact_root.starts_with(report_root)
        || archive_root.starts_with(shard_artifact_root)
        || shard_artifact_root.starts_with(archive_root)
        || !runtime_dependency_report.starts_with(report_root)
        || runtime_dependency_report == report_root
        || runtime_dependency_report.starts_with(archive_root)
        || runtime_dependency_report.starts_with(shard_artifact_root)
        || archive_root.starts_with(runtime_dependency_report)
        || shard_artifact_root.starts_with(runtime_dependency_report)
    {
        anyhow::bail!(
            "full shard archive, shard, and runtime dependency paths must be disjoint children of the report root"
        );
    }
    let mut runtime_dependency_ids = BTreeSet::new();
    for dependency in &contract.runtime_dependencies {
        if dependency.id.trim().is_empty()
            || dependency.executable.trim().is_empty()
            || dependency.version_args.is_empty()
            || dependency.version_args.iter().any(|arg| arg.trim().is_empty())
            || dependency.expected_version_line.trim().is_empty()
            || dependency.provisioning.trim().is_empty()
            || !runtime_dependency_ids.insert(dependency.id.as_str())
        {
            anyhow::bail!("full shard runtime dependency contract is incomplete");
        }
    }
    let denominator = contract.shards.len();
    let mut ids = BTreeSet::new();
    let mut indices = BTreeSet::new();
    for shard in &contract.shards {
        if shard.id.is_empty()
            || !shard.id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || !ids.insert(shard.id.as_str())
        {
            anyhow::bail!("full shard identity is invalid");
        }
        let value = shard
            .partition
            .strip_prefix("hash:")
            .context("full shard partition must use hash strategy")?;
        let (index, declared_denominator) =
            value.split_once('/').context("full shard partition is malformed")?;
        let index = index.parse::<usize>().context("full shard index is invalid")?;
        let declared_denominator =
            declared_denominator.parse::<usize>().context("full shard denominator is invalid")?;
        if declared_denominator != denominator
            || !(1..=denominator).contains(&index)
            || !indices.insert(index)
        {
            anyhow::bail!("full shard partitions do not cover the hash space exactly once");
        }
    }
    Ok(())
}

fn validate_full_shard_runtime_dependency_catalog(
    workspace: &Workspace,
    dependencies: &[FullShardRuntimeDependency],
) -> Result<()> {
    let catalog = bijux_dna_containers::api::load_container_catalog(&workspace.root)
        .context("load container catalog for full-shard runtime dependencies")?;
    for dependency in dependencies {
        let tool =
            catalog.tools.iter().find(|tool| tool.id == dependency.id).with_context(|| {
                format!(
                    "full-shard runtime dependency {} is absent from the container catalog",
                    dependency.id
                )
            })?;
        let expected_version_line = format!("{} {}", tool.expected_bin, tool.version);
        if dependency.executable != tool.expected_bin
            || dependency.expected_version_line != expected_version_line
        {
            anyhow::bail!(
                "full-shard runtime dependency {} drifted from container catalog executable/version identity",
                dependency.id
            );
        }
    }
    Ok(())
}

fn validate_fast_group_contract(contract: &FastGroupContract) -> Result<()> {
    if contract.schema_version != FAST_GROUPS_SCHEMA_V3
        || !(2..=16).contains(&contract.max_parallelism)
        || !(2..=16).contains(&contract.groups.len())
        || !(5..=100).contains(&contract.history_limit)
        || contract.warm_cache_target_milliseconds == 0
        || !(1..=10_000).contains(&contract.minimum_cache_hit_ratio_basis_points)
        || contract.reference_runner.os.is_empty()
        || contract.reference_runner.architecture.is_empty()
        || contract.reference_runner.rust_toolchain.is_empty()
        || contract.reference_runner.compiler_wrapper != "sccache"
    {
        anyhow::bail!("ci-fast group contract is incomplete");
    }
    validate_artifact_relative_path(&contract.report_root)?;
    validate_artifact_relative_path(&contract.timing_report)?;
    validate_artifact_relative_path(&contract.timing_history)?;
    validate_artifact_relative_path(&contract.baseline_report)?;
    if BTreeSet::from([
        contract.timing_report.as_str(),
        contract.timing_history.as_str(),
        contract.baseline_report.as_str(),
    ])
    .len()
        != 3
    {
        anyhow::bail!("ci-fast report paths must be distinct");
    }

    let excluded_lane_ids =
        contract.excluded_lanes.iter().map(|lane| lane.id.as_str()).collect::<BTreeSet<_>>();
    if excluded_lane_ids
        != BTreeSet::from(["full-workspace", "lunarc-integration", "nightly-release"])
        || excluded_lane_ids.len() != contract.excluded_lanes.len()
        || contract.excluded_lanes.iter().any(|lane| {
            lane.reason.trim().is_empty()
                || lane.evidence_command.trim().is_empty()
                || lane.evidence_command.contains(['\n', '\0'])
        })
    {
        anyhow::bail!("ci-fast excluded lanes are incomplete");
    }

    let mut ids = BTreeSet::new();
    for group in &contract.groups {
        if group.id.is_empty()
            || !group.id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || !ids.insert(group.id.as_str())
            || group.program != "make"
            || group.args.first().map(String::as_str) != Some("--no-print-directory")
            || group.args.iter().any(|arg| arg.is_empty() || arg.contains(['\n', '\0']))
        {
            anyhow::bail!("ci-fast group identity or command is invalid");
        }
    }
    for group in &contract.groups {
        let mut dependencies = BTreeSet::new();
        for dependency in &group.dependencies {
            if dependency == &group.id
                || !ids.contains(dependency.as_str())
                || !dependencies.insert(dependency.as_str())
            {
                anyhow::bail!("ci-fast group dependency is invalid");
            }
        }
    }
    let mut resolved = BTreeSet::new();
    while resolved.len() < contract.groups.len() {
        let ready = contract
            .groups
            .iter()
            .filter(|group| {
                !resolved.contains(group.id.as_str())
                    && group
                        .dependencies
                        .iter()
                        .all(|dependency| resolved.contains(dependency.as_str()))
            })
            .map(|group| group.id.as_str())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            anyhow::bail!("ci-fast group dependencies contain a cycle");
        }
        resolved.extend(ready);
    }
    Ok(())
}

fn load_fast_timing_history(path: &Path) -> Result<FastTimingHistory> {
    if !path.try_exists()? {
        return Ok(FastTimingHistory {
            schema_version: FAST_TIMING_HISTORY_SCHEMA_V4.to_string(),
            observations: Vec::new(),
        });
    }
    let mut history: FastTimingHistory = serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    if matches!(
        history.schema_version.as_str(),
        "bijux.ci.fast_timing_history.v1"
            | "bijux.ci.fast_timing_history.v2"
            | "bijux.ci.fast_timing_history.v3"
    ) {
        history.schema_version = FAST_TIMING_HISTORY_SCHEMA_V4.to_string();
    } else if history.schema_version != FAST_TIMING_HISTORY_SCHEMA_V4 {
        anyhow::bail!("unsupported ci-fast timing history schema");
    }
    Ok(history)
}

fn source_commit(workspace: &Workspace) -> Result<String> {
    let output = ProcessRunner::new(workspace).run(&["git", "rev-parse", "HEAD"])?;
    if !output.status.success() {
        anyhow::bail!("resolve ci-fast source commit");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn source_worktree_clean(workspace: &Workspace) -> Result<bool> {
    let output = ProcessRunner::new(workspace).run(&[
        "git",
        "status",
        "--porcelain=v1",
        "--untracked-files=all",
    ])?;
    if !output.status.success() {
        anyhow::bail!("resolve ci-fast source worktree state");
    }
    Ok(output.stdout.is_empty())
}

fn rust_toolchain_version(workspace: &Workspace) -> Result<String> {
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

fn compiler_wrapper_identity(workspace: &Workspace) -> Result<Option<String>> {
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

fn compiler_wrapper_program(workspace: &Workspace) -> Result<Option<String>> {
    Ok(artifact_env_with_common_test_env(workspace)?
        .into_iter()
        .rev()
        .find_map(|(key, value)| (key == "RUSTC_WRAPPER").then_some(value))
        .filter(|value| !value.trim().is_empty()))
}

fn parse_compiler_wrapper_identity(version: &str) -> Option<String> {
    version.split_whitespace().next().filter(|name| *name == "sccache").map(ToOwned::to_owned)
}

fn read_sccache_stats(
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

fn parse_sccache_stats(value: &serde_json::Value) -> Result<FastCacheStats> {
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

fn cache_stats_delta(
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

fn verified_warm_cache_mode(
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

fn duration_milliseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn percentile(values: &[u64], percentile: usize) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100).saturating_sub(1);
    ordered.get(rank).copied()
}

fn reference_timing_samples(
    history: &FastTimingHistory,
    reference: &FastReferenceRunner,
) -> Vec<u64> {
    history
        .observations
        .iter()
        .filter(|observation| {
            observation.os == reference.os
                && observation.architecture == reference.architecture
                && observation.rust_toolchain == reference.rust_toolchain
                && observation.compiler_wrapper == reference.compiler_wrapper
                && observation.cache_state == "warm_verified"
                && !observation.cache_authority.is_empty()
                && !observation.build_authority.is_empty()
                && match observation.cache_reuse_mode.as_str() {
                    "build_target" => true,
                    "compiler_cache" | "" => observation.cache_hit_ratio_basis_points > 0,
                    _ => false,
                }
        })
        .map(|observation| observation.wall_time_milliseconds)
        .collect()
}

fn validate_artifact_relative_path(value: &str) -> Result<()> {
    let path = Path::new(value);
    if path.is_absolute()
        || !path.starts_with("artifacts")
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        anyhow::bail!("full shard output path must remain under artifacts/");
    }
    Ok(())
}

pub(in super::super) fn tooling_ci_test_slow(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-test-slow", args)?;
    set_assets_readonly(workspace, true)?;
    let envs = ci_test_env(workspace, true)?;
    let nextest_config =
        env_or_default("NEXTEST_CONFIG", "--config-file configs/rust/nextest.toml");
    let test_features = env_or_default("TEST_FEATURES", "--all-features");
    let no_tests = env_or_default("NEXTEST_NO_TESTS", "pass");
    let mut nextest_args = std::iter::once("nextest".to_string())
        .chain(std::iter::once("run".to_string()))
        .chain(nextest_config.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--workspace".to_string()))
        .chain(test_features.split_whitespace().map(ToOwned::to_owned))
        .chain(std::iter::once("--profile".to_string()))
        .chain(std::iter::once(
            std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "slow-integration".to_string()),
        ))
        .chain(std::iter::once("--test-threads".to_string()))
        .chain(std::iter::once(
            std::env::var("NEXTEST_TEST_THREADS").unwrap_or_else(|_| "8".to_string()),
        ))
        .chain(std::iter::once("--no-tests".to_string()))
        .chain(std::iter::once(no_tests))
        .collect::<Vec<_>>();
    nextest_args.extend(
        std::env::var("RUN_IGNORED")
            .unwrap_or_else(|_| "--run-ignored all".to_string())
            .split_whitespace()
            .map(ToOwned::to_owned),
    );
    nextest_args.push("-E".to_string());
    nextest_args.push(governed_slow_expression(workspace)?);
    let outcome = run_program_with_env(workspace, "cargo", &nextest_args, &envs);
    let restore = set_assets_readonly(workspace, false);
    let test_outcome = outcome?;
    restore?;
    Ok(test_outcome)
}

pub(in super::super) fn tooling_ci_coverage(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-coverage", args)?;
    let artifact_root = artifact_root_path(workspace)?;
    let coverage_root = artifact_root.join("coverage");
    if coverage_root.exists() {
        fs::remove_dir_all(&coverage_root)
            .with_context(|| format!("remove {}", coverage_root.display()))?;
    }
    bijux_dna_infra::ensure_dir(&coverage_root)
        .with_context(|| format!("create {}", coverage_root.display()))?;
    let envs = ci_test_env(workspace, false)?;
    let nextest_config =
        env_or_default("NEXTEST_CONFIG", "--config-file configs/rust/nextest.toml");
    let test_features = env_or_default("TEST_FEATURES", "--all-features");
    let nextest_profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".to_string());
    let nextest_threads = std::env::var("NEXTEST_TEST_THREADS").unwrap_or_else(|_| "1".to_string());
    let run_ignored = resolved_run_ignored(false)?;
    let no_cfg_coverage =
        read_coverage_runner_flag(workspace, "no_cfg_coverage", "--no-cfg-coverage")?;
    let coverage_out =
        std::env::var("COVERAGE_OUT").unwrap_or_else(|_| "coverage.json".to_string());
    let mut clean = run_program_with_env(
        workspace,
        "cargo",
        &["llvm-cov".to_string(), "clean".to_string()],
        &envs,
    )?;
    let nextest = run_program_with_env(
        workspace,
        "cargo",
        &std::iter::once("llvm-cov".to_string())
            .chain(std::iter::once("nextest".to_string()))
            .chain(std::iter::once("--no-report".to_string()))
            .chain(std::iter::once(no_cfg_coverage))
            .chain(nextest_config.split_whitespace().map(ToOwned::to_owned))
            .chain(std::iter::once("--workspace".to_string()))
            .chain(test_features.split_whitespace().map(ToOwned::to_owned))
            .chain(std::iter::once("--profile".to_string()))
            .chain(std::iter::once(nextest_profile))
            .chain(std::iter::once("--test-threads".to_string()))
            .chain(std::iter::once(nextest_threads))
            .chain(run_ignored.split_whitespace().map(ToOwned::to_owned))
            .collect::<Vec<_>>(),
        &envs,
    )?;
    clean = merge_outcomes(clean, nextest);
    if !clean.is_success() {
        return Ok(clean);
    }
    let json_report = run_program_with_env(
        workspace,
        "cargo",
        &[
            "llvm-cov".to_string(),
            "report".to_string(),
            "--json".to_string(),
            "--output-path".to_string(),
            coverage_root.join(&coverage_out).display().to_string(),
        ],
        &envs,
    )?;
    let html_report = run_program_with_env(
        workspace,
        "cargo",
        &[
            "llvm-cov".to_string(),
            "report".to_string(),
            "--html".to_string(),
            "--output-dir".to_string(),
            coverage_root.display().to_string(),
        ],
        &envs,
    )?;
    Ok(merge_outcomes(merge_outcomes(clean, json_report), html_report))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::Context;

    use super::{
        artifact_root_path, cache_stats_delta, cargo_lock_wait_events, ci_fast_critical_path,
        ci_fast_group_env, ci_output_path_under, full_archive_args, full_archive_roots,
        full_shard_args, full_shard_env, full_shard_execution_scope, full_shard_root_receipt,
        full_shard_roots, full_shard_storage_receipt, isolated_build_tree,
        materialize_ci_fast_compiler_wrapper, materialize_full_shard_workspace,
        observe_full_shard_runtime_dependencies, observe_full_shard_runtime_dependencies_with_env,
        parse_compiler_wrapper_identity, parse_sccache_stats, percentile, read_ci_build_authority,
        reference_timing_samples, remove_full_shard_ephemera, resolve_ci_fast_build_authority,
        resolve_ci_fast_build_authority_with_scratch_roots, resolve_ci_fast_sccache_runtime_env,
        resolve_full_shard_storage_roots, select_full_shards, set_full_archive_readonly,
        start_owned_sccache_server, validate_ci_test_executor_observation,
        validate_fast_group_contract, validate_full_archive_roots, validate_full_shard_contract,
        validate_full_shard_root_set, validate_full_shard_runtime_dependency_catalog,
        verified_warm_cache_mode, verify_sccache_compiler, FastCacheStats, FastExcludedLane,
        FastGroup, FastGroupContract, FastReferenceRunner, FastTimingHistory,
        FastTimingObservation, FullArchiveRoots, FullShard, FullShardContract, FullShardCoverage,
        FullShardEphemera, FullShardRoots, FullShardRuntimeDependency,
        FAST_TIMING_HISTORY_SCHEMA_V4, FULL_SHARD_SCRATCH_ROOT_ENV,
    };
    use crate::runtime::{process::ProcessRunner, workspace::Workspace};

    fn contract() -> FullShardContract {
        FullShardContract {
            schema_version: "bijux.ci.full_shards.v5".to_string(),
            coverage: FullShardCoverage {
                workspace: true,
                all_targets: true,
                all_features: true,
                run_ignored: "all".to_string(),
                overlap_policy: "exactly_once".to_string(),
            },
            partition_strategy: "hash".to_string(),
            workspace_isolation: "tracked_source_clones_with_shared_immutable_archive".to_string(),
            build_authority: "configs/ci/build-authority.json".to_string(),
            runtime_dependency_report: "artifacts/ci/full-shards/runtime-dependencies.json"
                .to_string(),
            runtime_dependencies: vec![FullShardRuntimeDependency {
                id: "bcftools".to_string(),
                executable: "bcftools".to_string(),
                version_args: vec!["--version".to_string()],
                expected_version_line: "bcftools 1.21".to_string(),
                provisioning: "caller-supplied PATH".to_string(),
            }],
            archive_build_jobs: 4,
            archive_root: "artifacts/ci/full-shards/archive".to_string(),
            shard_artifact_root: "artifacts/ci/full-shards/shards".to_string(),
            report_root: "artifacts/ci/full-shards".to_string(),
            shards: (1..=5)
                .map(|index| FullShard {
                    id: format!("hash-{index}-of-5"),
                    partition: format!("hash:{index}/5"),
                })
                .collect(),
        }
    }

    fn fast_contract() -> FastGroupContract {
        FastGroupContract {
            schema_version: "bijux.ci.fast_groups.v3".to_string(),
            max_parallelism: 2,
            warm_cache_target_milliseconds: 300_000,
            minimum_cache_hit_ratio_basis_points: 8_000,
            history_limit: 40,
            report_root: "artifacts/ci/ci-fast-groups".to_string(),
            timing_report: "artifacts/ci/ci-fast-timing.json".to_string(),
            timing_history: "artifacts/ci/ci-fast-timing-history.json".to_string(),
            baseline_report: "artifacts/ci/baseline-timing.json".to_string(),
            reference_runner: FastReferenceRunner {
                os: "linux".to_string(),
                architecture: "x86_64".to_string(),
                rust_toolchain: "1.95.0".to_string(),
                compiler_wrapper: "sccache".to_string(),
            },
            excluded_lanes: [
                ("full-workspace", "full", "make _ci-slow"),
                ("lunarc-integration", "heavy", "bijux-dna benchmark cycle advance"),
                ("nightly-release", "release", "make _ci-slow"),
            ]
            .into_iter()
            .map(|(id, reason, evidence_command)| FastExcludedLane {
                id: id.to_string(),
                reason: reason.to_string(),
                evidence_command: evidence_command.to_string(),
            })
            .collect(),
            groups: ["metadata", "unit"]
                .into_iter()
                .map(|id| FastGroup {
                    id: id.to_string(),
                    dependencies: Vec::new(),
                    program: "make".to_string(),
                    args: vec!["--no-print-directory".to_string(), format!("_{id}")],
                })
                .collect(),
        }
    }

    #[test]
    fn full_shard_contract_covers_hash_space_exactly_once() {
        assert!(validate_full_shard_contract(&contract()).is_ok());
    }

    #[test]
    fn full_shard_selection_supports_exact_affected_recovery() -> anyhow::Result<()> {
        let contract = contract();
        assert_eq!(select_full_shards(&contract, &[])?.len(), 5);

        let selected = select_full_shards(
            &contract,
            &[
                "--shard".to_string(),
                "hash-4-of-5".to_string(),
                "--shard".to_string(),
                "hash-5-of-5".to_string(),
            ],
        )?;
        assert_eq!(
            selected.iter().map(|shard| shard.id.as_str()).collect::<Vec<_>>(),
            ["hash-4-of-5", "hash-5-of-5"]
        );
        let scope = full_shard_execution_scope(&contract, &selected, false);
        assert_eq!(scope["mode"], "affected_recovery");
        assert_eq!(scope["complete_contract_coverage"], false);
        assert_eq!(scope["selected_shard_count"], 2);
        assert!(select_full_shards(
            &contract,
            &[
                "--shard".to_string(),
                "hash-4-of-5".to_string(),
                "--shard".to_string(),
                "hash-4-of-5".to_string(),
            ]
        )
        .is_err());
        assert!(select_full_shards(&contract, &["--shard".to_string(), "hash-9-of-5".to_string()])
            .is_err());
        assert!(select_full_shards(&contract, &["--shard".to_string()]).is_err());
        Ok(())
    }

    #[test]
    fn checked_in_full_shard_contract_uses_five_complete_partitions() -> anyhow::Result<()> {
        let repository_root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let path = repository_root.join("configs/ci/full-shards.json");
        let contract: FullShardContract = serde_json::from_slice(&std::fs::read(path)?)?;

        validate_full_shard_contract(&contract)?;
        validate_full_shard_runtime_dependency_catalog(
            &Workspace { root: repository_root },
            &contract.runtime_dependencies,
        )?;
        assert_eq!(contract.shards.len(), 5);
        assert_eq!(
            contract.shards.iter().map(|shard| shard.partition.as_str()).collect::<Vec<_>>(),
            ["hash:1/5", "hash:2/5", "hash:3/5", "hash:4/5", "hash:5/5"]
        );
        assert_eq!(
            contract.runtime_dependencies[0].provisioning,
            "checksum-bound runner execution binding; site modules are not required"
        );
        Ok(())
    }

    #[test]
    fn full_shards_publish_runtime_dependency_preflight_receipts() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "bcftools".to_string(),
            executable: "sh".to_string(),
            version_args: vec!["-c".to_string(), "printf 'bcftools 1.21\\n'".to_string()],
            expected_version_line: "bcftools 1.21".to_string(),
            provisioning: "test PATH".to_string(),
        }];

        let observations =
            observe_full_shard_runtime_dependencies(&workspace, &dependencies, &report_path)?;
        assert_eq!(observations.len(), 1);
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
        assert_eq!(receipt.get("all_available").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            receipt.pointer("/dependencies/0/observed").and_then(serde_json::Value::as_str),
            Some("bcftools 1.21")
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn full_shard_preflight_honors_checksum_bound_runtime_dependencies() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt as _;

        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let executable = root.path().join("bound-bcftools");
        std::fs::write(&executable, b"#!/bin/sh\nprintf 'bcftools 1.21\\n'\n")?;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))?;
        let manifest_path = root.path().join("execution-bindings.json");
        let manifest = bijux_dna_runner::api::ExecutionBindingManifestV1 {
            schema_version: bijux_dna_runner::api::RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
            bindings: vec![bijux_dna_runner::api::CommandExecutionBindingV1 {
                command: "bijux-unavailable-bcftools".to_string(),
                program: executable.display().to_string(),
                prefix_args: Vec::new(),
                required_files: vec![bijux_dna_runner::api::BoundFileIdentityV1 {
                    path: executable.display().to_string(),
                    sha256: bijux_dna_infra::hash_file_sha256(&executable)?,
                    size_bytes: executable.metadata()?.len(),
                }],
            }],
        };
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        let execution_env = BTreeMap::from([(
            bijux_dna_runner::api::RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "bcftools".to_string(),
            executable: "bijux-unavailable-bcftools".to_string(),
            version_args: vec!["--version".to_string()],
            expected_version_line: "bcftools 1.21".to_string(),
            provisioning: "checksum-bound runner execution binding".to_string(),
        }];

        let observations = observe_full_shard_runtime_dependencies_with_env(
            &workspace,
            &dependencies,
            &report_path,
            Some(&execution_env),
        )?;

        assert_eq!(observations.len(), 1);
        assert_eq!(
            observations[0].get("available").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            observations[0].get("observed").and_then(serde_json::Value::as_str),
            Some("bcftools 1.21")
        );
        Ok(())
    }

    #[test]
    fn full_shards_stop_before_launch_when_runtime_dependencies_are_missing() -> anyhow::Result<()>
    {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "missing-tool".to_string(),
            executable: "bijux-missing-full-shard-runtime-tool".to_string(),
            version_args: vec!["--version".to_string()],
            expected_version_line: "missing-tool 1.0".to_string(),
            provisioning: "test PATH".to_string(),
        }];

        let error =
            observe_full_shard_runtime_dependencies(&workspace, &dependencies, &report_path)
                .err()
                .context("a missing runtime dependency must stop full-shard launch")?;
        assert!(error.to_string().contains("runtime dependencies are unavailable"));
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
        assert_eq!(receipt.get("all_available").and_then(serde_json::Value::as_bool), Some(false));
        assert_eq!(
            receipt.pointer("/dependencies/0/available").and_then(serde_json::Value::as_bool),
            Some(false)
        );
        Ok(())
    }

    #[test]
    fn full_shards_refuse_missing_or_drifted_test_executor_before_launch() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let authority =
            read_ci_build_authority(&workspace.path("configs/ci/build-authority.json"))?;

        let missing = validate_ci_test_executor_observation(&authority, false, "no such command")
            .err()
            .ok_or_else(|| anyhow::anyhow!("missing nextest did not fail closed"))?;
        assert!(missing.to_string().contains("before launching shards"));

        let drifted =
            validate_ci_test_executor_observation(&authority, true, "cargo-nextest 0.9.99")
                .err()
                .ok_or_else(|| anyhow::anyhow!("drifted nextest did not fail closed"))?;
        assert!(drifted.to_string().contains("version mismatch"));

        validate_ci_test_executor_observation(&authority, true, "cargo-nextest 0.9.100 (build)")
    }

    #[test]
    fn full_shard_contract_rejects_overlap_and_escaping_outputs() {
        let mut overlap = contract();
        overlap.shards[4].partition = "hash:4/5".to_string();
        assert!(validate_full_shard_contract(&overlap).is_err());

        let mut escaping = contract();
        escaping.report_root = "../reports".to_string();
        assert!(validate_full_shard_contract(&escaping).is_err());

        let mut shared_workspace = contract();
        shared_workspace.workspace_isolation = "shared".to_string();
        assert!(validate_full_shard_contract(&shared_workspace).is_err());

        let mut shared_outputs = contract();
        shared_outputs.shard_artifact_root = shared_outputs.report_root.clone();
        assert!(validate_full_shard_contract(&shared_outputs).is_err());

        let mut shared_archive = contract();
        shared_archive.archive_root = shared_archive.shard_artifact_root.clone();
        assert!(validate_full_shard_contract(&shared_archive).is_err());

        let mut unbounded_archive = contract();
        unbounded_archive.archive_build_jobs = 0;
        assert!(validate_full_shard_contract(&unbounded_archive).is_err());

        let mut unknown_authority = contract();
        unknown_authority.build_authority = "configs/ci/another-authority.json".to_string();
        assert!(validate_full_shard_contract(&unknown_authority).is_err());
    }

    #[test]
    fn full_shard_runtime_roots_are_absolute_complete_disjoint_and_read_only() -> anyhow::Result<()>
    {
        let workspace = Workspace::resolve()?;
        let contract = contract();
        let storage = resolve_full_shard_storage_roots(&workspace, &contract, None)?;
        let roots = contract
            .shards
            .iter()
            .map(|shard| full_shard_roots(&storage, &contract, shard))
            .collect::<anyhow::Result<Vec<_>>>()?;
        validate_full_shard_root_set(&storage, &roots)?;

        let shard_workspace = storage.scratch_root.join("workspaces/hash-1-of-5");
        std::fs::create_dir_all(&shard_workspace)?;
        let first_env = full_shard_env(&roots[0], &shard_workspace)?
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        for name in [
            "ARTIFACT_ROOT",
            "ISO_ROOT",
            "CARGO_TARGET_DIR",
            "TEST_TARGET_DIR",
            "COV_TARGET_DIR",
            "TEST_TMP_DIR",
            "COV_TMP_DIR",
            "TEST_PROFRAW_DIR",
            "COV_PROFRAW_DIR",
        ] {
            let value = first_env
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("missing shard environment variable {name}"))?;
            let path = std::path::Path::new(value);
            assert!(path.is_absolute());
            assert!(path.starts_with(&storage.scratch_root));
        }
        assert_ne!(first_env["CARGO_TARGET_DIR"], first_env["TEST_TARGET_DIR"]);
        assert_ne!(first_env["TEST_TARGET_DIR"], first_env["COV_TARGET_DIR"]);
        assert_ne!(first_env["TEST_TMP_DIR"], first_env["COV_TMP_DIR"]);
        assert_ne!(first_env["TEST_PROFRAW_DIR"], first_env["COV_PROFRAW_DIR"]);
        assert_eq!(first_env["BIJUX_REPO_ROOT"], shard_workspace.display().to_string());
        assert_eq!(
            first_env["BIJUX_BENCHMARK_ROOT"],
            shard_workspace.join("benchmarks").display().to_string()
        );
        let consumer = Workspace { root: shard_workspace.clone() };
        let outputs = std::path::Path::new(&first_env["ARTIFACT_ROOT"]);
        assert_eq!(consumer.admit_artifact_path(outputs)?, consumer.path("artifacts"));
        assert_eq!(first_env["ISO_ROOT"], first_env["ARTIFACT_ROOT"]);
        assert!(consumer.admit_artifact_path(&roots[0].artifact_root).is_err());
        assert_eq!(first_env["INSTA_UPDATE"], "no");
        assert!(!first_env.contains_key("INSTA_WORKSPACE_ROOT"));
        let archive = full_archive_roots(&storage, &contract)?;
        let receipt = full_shard_root_receipt(&roots[0], &archive, &shard_workspace);
        assert_eq!(receipt["repository_root"], first_env["BIJUX_REPO_ROOT"]);
        assert_eq!(receipt["benchmark_root"], first_env["BIJUX_BENCHMARK_ROOT"]);
        assert_eq!(receipt["snapshot_source_authority"], "archive_compile_time_metadata");
        assert_eq!(receipt["snapshot_update_policy"], first_env["INSTA_UPDATE"]);
        assert_eq!(receipt["artifact_root"], first_env["ARTIFACT_ROOT"]);
        assert_eq!(receipt["iso_root"], first_env["ISO_ROOT"]);
        assert_eq!(receipt["archive_extract_dir"], archive.archive_extract.display().to_string());
        assert_eq!(receipt["cargo_target_dir"], first_env["CARGO_TARGET_DIR"]);
        assert_eq!(receipt["test_target_dir"], first_env["TEST_TARGET_DIR"]);
        assert_eq!(receipt["coverage_target_dir"], first_env["COV_TARGET_DIR"]);
        assert_eq!(receipt["test_tmp_dir"], first_env["TEST_TMP_DIR"]);
        assert_eq!(receipt["coverage_tmp_dir"], first_env["COV_TMP_DIR"]);
        assert_eq!(receipt["test_profraw_dir"], first_env["TEST_PROFRAW_DIR"]);
        assert_eq!(receipt["coverage_profraw_dir"], first_env["COV_PROFRAW_DIR"]);
        let storage_receipt = full_shard_storage_receipt(&storage);
        assert_eq!(storage_receipt["scratch_root"], storage.scratch_root.display().to_string());
        assert_eq!(
            storage_receipt["retained_report_root"],
            storage.retained_report_root.display().to_string()
        );
        assert_eq!(storage_receipt["scratch_root_environment"], FULL_SHARD_SCRATCH_ROOT_ENV);
        Ok(())
    }

    #[test]
    fn full_shards_build_once_and_execute_only_from_the_archive() -> anyhow::Result<()> {
        let workspace = Workspace::resolve()?;
        let contract = contract();
        let storage = resolve_full_shard_storage_roots(&workspace, &contract, None)?;
        let archive = full_archive_roots(&storage, &contract)?;
        validate_full_archive_roots(&storage, &archive)?;

        let build_args = full_archive_args(&archive.archive_file);
        for required in ["archive", "--workspace", "--all-targets", "--all-features", "--locked"] {
            assert!(build_args.iter().any(|arg| arg == required));
        }
        assert_eq!(build_args.iter().filter(|arg| arg.as_str() == "--archive-file").count(), 1);

        let inventory = super::full_inventory_args(&archive);
        assert!(inventory.windows(2).any(|pair| pair[0] == "--extract-to"
            && pair[1] == archive.archive_extract.display().to_string()));
        assert!(!inventory.iter().any(|arg| arg == "--workspace" || arg == "--all-targets"));

        for shard in &contract.shards {
            let remap = storage.scratch_root.join("workspaces").join(&shard.id);
            let args = full_shard_args(shard, &archive.archive_extract, &remap)?;
            for forbidden in [
                "--workspace",
                "--all-targets",
                "--all-features",
                "--locked",
                "--archive-file",
                "--extract-to",
            ] {
                assert!(!args.iter().any(|arg| arg == forbidden));
            }
            assert!(args.windows(2).any(|pair| {
                pair[0] == "--cargo-metadata"
                    && pair[1]
                        == archive
                            .archive_extract
                            .join("target/nextest/cargo-metadata.json")
                            .display()
                            .to_string()
            }));
            assert!(args.windows(2).any(|pair| {
                pair[0] == "--target-dir-remap"
                    && pair[1] == archive.archive_extract.join("target").display().to_string()
            }));
            assert!(args.windows(2).any(|pair| {
                pair[0] == "--workspace-remap" && pair[1] == remap.display().to_string()
            }));
        }
        Ok(())
    }

    #[test]
    fn shared_archive_preparation_creates_an_empty_extraction_destination() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let archive = temp.path().join("archive/extracted");
        super::prepare_full_archive_extraction(&archive)?;
        assert!(archive.is_dir());
        std::fs::write(archive.join("stale"), b"old generated artifact")?;
        super::seal_full_archive_extraction(&archive)?;
        super::prepare_full_archive_extraction(&archive)?;
        assert_eq!(std::fs::read_dir(&archive)?.count(), 0);
        Ok(())
    }

    #[test]
    fn shared_archive_sealing_preserves_bytes_and_owned_cleanup() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let archive = temp.path().join("extracted");
        let member = archive.join("target/nextest/binaries-metadata.json");
        std::fs::create_dir_all(member.parent().context("member parent")?)?;
        std::fs::write(&member, b"governed archive metadata")?;
        super::seal_full_archive_extraction(&archive)?;
        assert_eq!(std::fs::read(&member)?, b"governed archive metadata");
        assert!(std::fs::metadata(&member)?.permissions().readonly());
        assert!(std::fs::metadata(&archive)?.permissions().readonly());
        super::remove_full_shard_ephemera(&archive)?;
        assert!(!archive.exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn shared_archive_sealing_never_follows_external_symlinks() -> anyhow::Result<()> {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let temp = tempfile::tempdir()?;
        let outside = temp.path().join("outside");
        let archive = temp.path().join("extracted");
        std::fs::create_dir(&outside)?;
        std::fs::create_dir(&archive)?;
        let file = outside.join("writable");
        std::fs::write(&file, b"outside authority")?;
        let mode = std::fs::metadata(&file)?.permissions().mode();
        symlink(&outside, archive.join("linked"))?;
        super::seal_full_archive_extraction(&archive)?;
        assert_eq!(std::fs::metadata(&file)?.permissions().mode(), mode);
        assert_eq!(std::fs::read(&file)?, b"outside authority");
        assert!(super::seal_full_archive_extraction(&archive.join("linked")).is_err());
        super::remove_full_shard_ephemera(&archive)?;
        assert!(file.is_file());
        Ok(())
    }

    #[test]
    fn ci_outputs_rebase_under_configured_artifact_authority() -> anyhow::Result<()> {
        let authority = std::path::Path::new("/local/slurmtmp.1234/bijux/artifacts");

        let archive = ci_output_path_under(authority, "artifacts/ci/full-shards/archive")?;
        let report = ci_output_path_under(authority, "artifacts/ci/full-gate-timing.json")?;

        assert_eq!(archive, authority.join("ci/full-shards/archive"));
        assert_eq!(report, authority.join("ci/full-gate-timing.json"));
        assert!(ci_output_path_under(authority, "../outside").is_err());
        Ok(())
    }

    #[test]
    fn full_shard_scratch_authority_is_external_disjoint_and_fail_closed() -> anyhow::Result<()> {
        let workspace = Workspace::resolve()?;
        let contract = contract();
        let external = std::path::PathBuf::from("/local/slurmtmp.1234/bijux-full-shards");
        let storage =
            resolve_full_shard_storage_roots(&workspace, &contract, Some(external.clone()))?;

        assert_eq!(storage.scratch_root, external);
        assert!(storage.retained_report_root.starts_with(artifact_root_path(&workspace)?));
        assert!(!storage.scratch_root.starts_with(&storage.retained_report_root));
        assert!(!storage.retained_report_root.starts_with(&storage.scratch_root));

        let archive = full_archive_roots(&storage, &contract)?;
        validate_full_archive_roots(&storage, &archive)?;
        assert!(archive.artifact_root.starts_with(&storage.scratch_root));
        assert!(archive.build_log.starts_with(&storage.retained_report_root));

        assert!(resolve_full_shard_storage_roots(
            &workspace,
            &contract,
            Some(std::path::PathBuf::from("relative/scratch")),
        )
        .is_err());
        assert!(resolve_full_shard_storage_roots(
            &workspace,
            &contract,
            Some(storage.retained_report_root.join("scratch")),
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn full_shard_archive_is_read_only_during_partition_execution() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let artifact_root = root.join("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let archive_root = tempfile::tempdir_in(&artifact_root)?;
        let archive = archive_root.path().join("workspace-tests.tar.zst");
        std::fs::write(&archive, b"checksum-bound archive")?;

        set_full_archive_readonly(&archive)?;

        assert!(std::fs::metadata(&archive)?.permissions().readonly());
        Ok(())
    }

    #[test]
    fn full_shards_retain_only_bounded_logs_and_receipts() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let report_root = root.path().join("retained/full-shards");
        let scratch_root = root.path().join("scratch/full-shards");
        let archive_root = scratch_root.join("archive");
        let shard_root = scratch_root.join("shards/hash-1-of-5");
        std::fs::create_dir_all(scratch_root.join("workspaces/hash-1-of-5"))?;
        std::fs::create_dir_all(isolated_build_tree(&archive_root, "cargo"))?;
        std::fs::create_dir_all(&shard_root)?;
        std::fs::create_dir_all(&report_root)?;
        std::fs::write(report_root.join("hash-1-of-5.log"), b"test failure\n")?;
        std::fs::write(report_root.join("full-gate-timing.json"), b"{}\n")?;
        std::fs::write(report_root.join("archive-build.log"), b"archive build\n")?;
        std::fs::write(archive_root.join("workspace-tests.tar.zst"), b"archive\n")?;
        std::fs::write(
            isolated_build_tree(&archive_root, "cargo").join("output"),
            b"cargo output\n",
        )?;
        std::fs::write(shard_root.join("output"), b"shard output\n")?;
        let extracted = shard_root.join("nextest/archive/read-only");
        std::fs::create_dir_all(&extracted)?;
        let extracted_file = extracted.join("test-binary");
        std::fs::write(&extracted_file, b"nextest archive member\n")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&extracted_file, std::fs::Permissions::from_mode(0o400))?;
            std::fs::set_permissions(&extracted, std::fs::Permissions::from_mode(0o500))?;
            std::fs::set_permissions(
                shard_root.join("nextest/archive"),
                std::fs::Permissions::from_mode(0o500),
            )?;
            std::fs::set_permissions(
                shard_root.join("nextest"),
                std::fs::Permissions::from_mode(0o500),
            )?;
            std::fs::set_permissions(&shard_root, std::fs::Permissions::from_mode(0o500))?;
        }

        let archive_roots = FullArchiveRoots {
            artifact_root: archive_root.clone(),
            archive_extract: archive_root.join("extracted"),
            cargo_target: isolated_build_tree(&archive_root, "cargo"),
            archive_file: archive_root.join("workspace-tests.tar.zst"),
            build_log: report_root.join("archive-build.log"),
        };
        let shard_roots = vec![FullShardRoots {
            artifact_root: shard_root.clone(),
            cargo_target: isolated_build_tree(&shard_root, "cargo"),
            test_target: isolated_build_tree(&shard_root, "test"),
            coverage_target: isolated_build_tree(&shard_root, "coverage"),
            test_tmp: shard_root.join("tmp/test"),
            coverage_tmp: shard_root.join("tmp/coverage"),
            test_profraw: shard_root.join("coverage/profraw-test"),
            coverage_profraw: shard_root.join("coverage/profraw-coverage"),
        }];
        let mut ephemera = FullShardEphemera::new(&scratch_root, &archive_roots, &shard_roots);
        ephemera.retain_bounded_evidence()?;

        assert!(!scratch_root.join("workspaces").exists());
        assert!(!archive_root.exists());
        assert!(!shard_root.exists());
        assert!(report_root.join("archive-build.log").is_file());
        assert!(report_root.join("hash-1-of-5.log").is_file());
        assert!(report_root.join("full-gate-timing.json").is_file());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn full_shard_ephemera_removes_nested_symlinks_without_following() -> anyhow::Result<()> {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir()?;
        let ephemeral = root.path().join("ephemeral");
        let retained = root.path().join("retained");
        std::fs::create_dir_all(&ephemeral)?;
        std::fs::write(&retained, b"retained evidence\n")?;
        symlink(&retained, ephemeral.join("escape"))?;

        remove_full_shard_ephemera(&ephemeral)?;

        assert_eq!(std::fs::read(&retained)?, b"retained evidence\n");
        assert!(!ephemeral.exists());
        Ok(())
    }

    #[test]
    fn full_shard_workspace_is_an_isolated_tracked_source_clone() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let artifact_root = workspace.path("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let report_root = tempfile::tempdir_in(&artifact_root)?;
        let shard =
            FullShard { id: "isolation-proof".to_string(), partition: "hash:1/5".to_string() };

        let snapshot = crate::runtime::source_snapshot::TrackedSourceSnapshot::capture(&workspace)?;
        let isolated = materialize_full_shard_workspace(&snapshot, report_root.path(), &shard.id)?;
        assert!(isolated.join(".git").is_dir());
        assert!(isolated.join("artifacts").is_dir());
        let root_manifest = std::fs::read(workspace.path("Cargo.toml"))?;
        assert_eq!(std::fs::read(isolated.join("Cargo.toml"))?, root_manifest);
        std::fs::write(isolated.join("Cargo.toml"), b"isolated mutation\n")?;
        assert_eq!(std::fs::read(workspace.path("Cargo.toml"))?, root_manifest);
        Ok(())
    }

    #[test]
    fn fast_group_contract_is_bounded_and_acyclic() {
        assert!(validate_fast_group_contract(&fast_contract()).is_ok());

        let mut cycle = fast_contract();
        cycle.groups[0].dependencies = vec!["unit".to_string()];
        cycle.groups[1].dependencies = vec!["metadata".to_string()];
        assert!(validate_fast_group_contract(&cycle).is_err());

        let mut escaping = fast_contract();
        escaping.timing_report = "../ci-fast.json".to_string();
        assert!(validate_fast_group_contract(&escaping).is_err());

        let mut omitted_lane = fast_contract();
        omitted_lane.excluded_lanes.pop();
        assert!(validate_fast_group_contract(&omitted_lane).is_err());

        let mut duplicate_lane = fast_contract();
        duplicate_lane.excluded_lanes.push(duplicate_lane.excluded_lanes[0].clone());
        assert!(validate_fast_group_contract(&duplicate_lane).is_err());

        let mut colliding_report = fast_contract();
        colliding_report.baseline_report = colliding_report.timing_report.clone();
        assert!(validate_fast_group_contract(&colliding_report).is_err());
    }

    #[test]
    fn cargo_lock_wait_measurement_counts_diagnostics_without_inference() {
        let log = b"Blocking waiting for file lock on build directory\n\
                    Compiling example v0.1.0\n\
                    blocking waiting for file lock on package cache\n";
        assert_eq!(cargo_lock_wait_events(log), 2);
        assert_eq!(cargo_lock_wait_events(b"Compiling without contention\n"), 0);
    }

    #[test]
    fn critical_path_accumulates_dependency_chain_durations() {
        let groups = vec![
            FastGroup {
                id: "source".to_string(),
                dependencies: Vec::new(),
                program: "true".to_string(),
                args: Vec::new(),
            },
            FastGroup {
                id: "dependent".to_string(),
                dependencies: vec!["source".to_string()],
                program: "true".to_string(),
                args: Vec::new(),
            },
            FastGroup {
                id: "independent".to_string(),
                dependencies: Vec::new(),
                program: "true".to_string(),
                args: Vec::new(),
            },
        ];
        let durations = BTreeMap::from([
            ("source".to_string(), 40),
            ("dependent".to_string(), 30),
            ("independent".to_string(), 60),
        ]);

        assert_eq!(
            ci_fast_critical_path(&groups, &durations),
            Some((vec!["source".to_string(), "dependent".to_string()], 70))
        );
    }

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
    fn timing_percentiles_use_nearest_rank() {
        assert_eq!(percentile(&[], 95), None);
        assert_eq!(percentile(&[100, 20, 40, 60, 80], 50), Some(60));
        assert_eq!(percentile(&[100, 20, 40, 60, 80], 95), Some(100));
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

    #[test]
    fn timing_samples_require_exact_warm_reference_runner() {
        let reference = fast_contract().reference_runner;
        let observation = |os: &str,
                           architecture: &str,
                           rust_toolchain: &str,
                           compiler_wrapper: &str,
                           cache_state: &str| {
            FastTimingObservation {
                observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
                source_commit: "0123456789abcdef".to_string(),
                os: os.to_string(),
                architecture: architecture.to_string(),
                rust_toolchain: rust_toolchain.to_string(),
                compiler_wrapper: compiler_wrapper.to_string(),
                cache_state: cache_state.to_string(),
                cache_authority: if cache_state == "warm_verified" {
                    "/shared/sccache".to_string()
                } else {
                    String::new()
                },
                build_authority: if cache_state == "warm_verified" {
                    "/shared/ci-fast-target".to_string()
                } else {
                    String::new()
                },
                cache_hit_ratio_basis_points: if cache_state == "warm_verified" {
                    9_000
                } else {
                    0
                },
                cache_reuse_mode: if cache_state == "warm_verified" {
                    "compiler_cache".to_string()
                } else {
                    String::new()
                },
                wall_time_milliseconds: 42,
            }
        };
        let mut reused_build_target =
            observation("linux", "x86_64", "1.95.0", "sccache", "warm_verified");
        reused_build_target.cache_hit_ratio_basis_points = 0;
        reused_build_target.cache_reuse_mode = "build_target".to_string();
        reused_build_target.wall_time_milliseconds = 43;
        let history = FastTimingHistory {
            schema_version: FAST_TIMING_HISTORY_SCHEMA_V4.to_string(),
            observations: vec![
                observation("linux", "x86_64", "1.95.0", "sccache", "warm_verified"),
                reused_build_target,
                observation("linux", "x86_64", "1.95.0", "sccache", "warm"),
                observation("macos", "aarch64", "1.95.0", "sccache", "warm_verified"),
                observation("linux", "x86_64", "1.94.0", "sccache", "warm_verified"),
                observation("linux", "x86_64", "1.95.0", "sccache", "cold"),
                observation("linux", "x86_64", "1.95.0", "", "warm_verified"),
            ],
        };

        assert_eq!(reference_timing_samples(&history, &reference), vec![42, 43]);
    }
    #[test]
    fn full_shard_receipt_includes_owned_cleanup_in_terminal_timing() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let archive = root.path().join("archive");
        std::fs::create_dir_all(archive.join("build"))?;
        std::fs::write(archive.join("build/product"), b"owned compilation output")?;
        let mut ephemera = FullShardEphemera {
            scratch_workspaces: root.path().join("workspaces"),
            archive_root: archive.clone(),
            shard_artifact_roots: vec![],
            armed: true,
        };
        let started = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(25))
            .ok_or_else(|| anyhow::anyhow!("cannot construct elapsed timer"))?;
        let path = root.path().join("timing.json");
        super::finish_full_shard_receipt(
            &path,
            serde_json::json!({"state":"complete","all_passed":true,"wall_time_milliseconds":0}),
            started,
            &mut ephemera,
        )?;
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert!(!archive.exists());
        assert!(!ephemera.armed);
        assert_eq!(receipt["all_passed"], true);
        assert_eq!(receipt["cleanup"]["completed"], true);
        assert_eq!(receipt["evidence_retention"]["build_targets_retained"], false);
        assert!(receipt["wall_time_milliseconds"].as_u64().is_some_and(|elapsed| elapsed >= 25));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn full_shard_receipt_records_cleanup_failure_and_preserves_external_targets(
    ) -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let external = root.path().join("retained");
        std::fs::create_dir_all(&external)?;
        std::fs::write(external.join("evidence"), b"retained")?;
        let archive = root.path().join("archive");
        std::os::unix::fs::symlink(&external, &archive)?;
        let mut ephemera = FullShardEphemera {
            scratch_workspaces: root.path().join("workspaces"),
            archive_root: archive,
            shard_artifact_roots: vec![],
            armed: true,
        };
        let path = root.path().join("timing.json");
        assert!(super::finish_full_shard_receipt(
            &path,
            serde_json::json!({"state":"complete","all_passed":true}),
            std::time::Instant::now(),
            &mut ephemera
        )
        .is_err());
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(receipt["all_passed"], false);
        assert_eq!(receipt["cleanup"]["completed"], false);
        assert_eq!(receipt["evidence_retention"]["archive_retained"], true);
        assert!(!ephemera.armed);
        assert_eq!(std::fs::read(external.join("evidence"))?, b"retained");
        Ok(())
    }
    #[test]
    fn full_shard_cleanup_retains_exclusively_owned_compilation_products() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let lease = super::CompilationCache::acquire(root.path())?;
        let product = lease.target().join("compiled-product");
        std::fs::write(&product, b"reusable")?;
        let mut ephemera = FullShardEphemera {
            scratch_workspaces: root.path().join("workspaces"),
            archive_root: root.path().join("archive"),
            shard_artifact_roots: vec![root.path().join("shards/hash-1-of-5")],
            armed: true,
        };
        std::fs::create_dir_all(&ephemera.archive_root)?;
        let report = root.path().join("timing.json");
        super::finish_full_shard_receipt(
            &report,
            serde_json::json!({"all_passed":true,"evidence_retention":{"compilation_cache_target":lease.target()}}),
            std::time::Instant::now(),
            &mut ephemera,
        )?;
        assert!(!ephemera.archive_root.exists());
        assert_eq!(std::fs::read(product)?, b"reusable");
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report)?)?;
        assert_eq!(receipt["evidence_retention"]["build_targets_retained"], true);
        assert_eq!(receipt["all_passed"], true);
        Ok(())
    }
    #[test]
    fn full_shard_archive_output_root_is_prepared_independently_of_the_cache() -> anyhow::Result<()>
    {
        let root = tempfile::tempdir()?;
        let storage = super::FullShardStorageRoots {
            retained_report_root: root.path().join("reports"),
            scratch_root: root.path().join("scratch"),
        };
        let roots = full_archive_roots(&storage, &contract())?;
        super::prepare_full_archive_roots(&roots)?;
        assert!(!roots.cargo_target.starts_with(&roots.artifact_root));
        assert!(roots.cargo_target.is_dir());
        assert!(roots.artifact_root.is_dir());
        std::fs::write(&roots.archive_file, b"archive output")?;
        Ok(())
    }
}
