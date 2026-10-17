//! Fast CI workflow and qualified reference evidence.
mod compiler_cache;
mod contract;
mod execution;
mod timing;

use crate::commands::ops::tooling::ci::ci_output_path;
use crate::commands::ops::tooling::ci::duration_milliseconds;
use crate::commands::ops::tooling::ci::fast::compiler_cache::cache_stats_delta;
use crate::commands::ops::tooling::ci::fast::compiler_cache::ci_fast_build_authority;
use crate::commands::ops::tooling::ci::fast::compiler_cache::compiler_wrapper_identity;
use crate::commands::ops::tooling::ci::fast::compiler_cache::compiler_wrapper_program;
use crate::commands::ops::tooling::ci::fast::compiler_cache::materialize_ci_fast_compiler_wrapper;
use crate::commands::ops::tooling::ci::fast::compiler_cache::read_sccache_stats;
use crate::commands::ops::tooling::ci::fast::compiler_cache::resolve_ci_fast_sccache_runtime_env;
use crate::commands::ops::tooling::ci::fast::compiler_cache::rust_toolchain_version;
use crate::commands::ops::tooling::ci::fast::compiler_cache::start_owned_sccache_server;
use crate::commands::ops::tooling::ci::fast::compiler_cache::verified_warm_cache_mode;
use crate::commands::ops::tooling::ci::fast::compiler_cache::verify_sccache_compiler;
use crate::commands::ops::tooling::ci::fast::contract::validate_fast_group_contract;
use crate::commands::ops::tooling::ci::fast::contract::FastGroupContract;
use crate::commands::ops::tooling::ci::fast::timing::ci_fast_critical_path;
use crate::commands::ops::tooling::ci::fast::timing::load_fast_timing_history;
use crate::commands::ops::tooling::ci::fast::timing::percentile;
use crate::commands::ops::tooling::ci::fast::timing::reference_timing_samples;
use crate::commands::ops::tooling::ci::fast::timing::FastTimingObservation;
use crate::commands::ops::tooling::ci::fast::timing::CI_CRITICAL_PATH_BASELINE_SCHEMA_V1;
use crate::commands::ops::tooling::ci::fast::timing::FAST_TIMING_SCHEMA_V5;
use crate::commands::ops::tooling::ci::source_worktree_clean;
use crate::commands::ops::tooling::ensure_help_only;
use crate::commands::ops::tooling::write_json_pretty;
use crate::model::ops::OpsCommandOutcome;
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

pub(in super::super::super) fn tooling_ci_fast(
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
    let execution::FastGroupExecution {
        passed,
        failed,
        receipts,
        stdout,
        stderr,
        group_durations,
        compiler_lock_wait_events,
        compiler_lock_wait_duration_upper_bound_milliseconds,
        compiler_lock_wait_targets,
    } = execution::run_groups(
        workspace,
        &contract,
        &report_root,
        &build_authority,
        &sccache_runtime_env,
        &group_compiler_env,
    )?;

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

pub(super) fn source_commit(workspace: &Workspace) -> Result<String> {
    let output = ProcessRunner::new(workspace).run(&["git", "rev-parse", "HEAD"])?;
    if !output.status.success() {
        anyhow::bail!("resolve ci-fast source commit");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
