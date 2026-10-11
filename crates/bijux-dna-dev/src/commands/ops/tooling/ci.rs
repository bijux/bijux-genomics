use super::super::suite_selection::governed_slow_expression;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use bijux_dna_policies::{validate_ci_build_authority, CiBuildAuthority};
use serde::{Deserialize, Serialize};

use super::{
    artifact_env, artifact_root_path, ci_test_env, ensure_help_only, env_or_default, fs,
    merge_outcomes, read_coverage_runner_flag, resolved_nextest_expression,
    resolved_nextest_profile, resolved_nextest_threads, resolved_run_ignored, run_check_ids,
    run_make_target, run_program_with_env, set_assets_readonly, write_json_pretty, Context,
    OpsCommandOutcome, Result, Workspace,
};
use crate::runtime::process::{ParallelProcessSpec, ProcessRunner};

const FULL_SHARDS_SCHEMA_V2: &str = "bijux.ci.full_shards.v2";
const FAST_GROUPS_SCHEMA_V1: &str = "bijux.ci.fast_groups.v1";
const FAST_TIMING_HISTORY_SCHEMA_V1: &str = "bijux.ci.fast_timing_history.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FullShardContract {
    schema_version: String,
    coverage: FullShardCoverage,
    partition_strategy: String,
    workspace_isolation: String,
    build_authority: String,
    shard_artifact_root: String,
    report_root: String,
    shards: Vec<FullShard>,
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastGroupContract {
    schema_version: String,
    max_parallelism: usize,
    warm_cache_target_milliseconds: u64,
    history_limit: usize,
    report_root: String,
    timing_report: String,
    timing_history: String,
    reference_runner: FastReferenceRunner,
    groups: Vec<FastGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastReferenceRunner {
    os: String,
    architecture: String,
    rust_toolchain: String,
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
    cache_state: String,
    wall_time_milliseconds: u64,
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

    let report_root = workspace.path(&contract.report_root);
    bijux_dna_infra::ensure_dir(&report_root)?;
    let envs = artifact_env(workspace)?;
    let started = Instant::now();
    let mut pending =
        contract.groups.iter().map(|group| group.id.as_str()).collect::<BTreeSet<_>>();
    let mut passed = BTreeSet::new();
    let mut failed = BTreeSet::new();
    let mut receipts = Vec::with_capacity(contract.groups.len());
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut critical_path = None::<(String, u64)>;

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
            stderr.push_str(&format!("ci-fast group {}: blocked by failed dependency\n", group.id));
            receipts.push(serde_json::json!({
                "id": group.id,
                "status": "blocked_dependency",
                "dependencies": group.dependencies,
                "duration_milliseconds": 0,
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
            .map(|group| ParallelProcessSpec {
                label: group.id.clone(),
                program: group.program.clone(),
                args: group.args.clone(),
                envs: envs.clone(),
                current_dir: None,
            })
            .collect();
        let outputs = ProcessRunner::new(workspace).run_parallel(specs)?;
        for (group, completed) in ready.into_iter().zip(outputs) {
            let log_path = report_root.join(format!("{}.log", group.id));
            let mut log = completed.output.stdout;
            log.extend_from_slice(&completed.output.stderr);
            bijux_dna_infra::write_bytes(&log_path, &log)?;
            let exit_code = completed.output.status.code().unwrap_or(1);
            let duration_milliseconds = duration_milliseconds(completed.duration);
            if critical_path.as_ref().is_none_or(|(_, duration)| duration_milliseconds > *duration)
            {
                critical_path = Some((group.id.clone(), duration_milliseconds));
            }
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
                "duration_milliseconds": duration_milliseconds,
                "exit_code": exit_code,
                "log": workspace.rel(&log_path).to_string_lossy(),
            }));
        }
    }

    let wall_time_milliseconds = duration_milliseconds(started.elapsed());
    let all_passed = failed.is_empty() && passed.len() == contract.groups.len();
    let source_commit = source_commit(workspace)?;
    let source_worktree_clean = source_worktree_clean(workspace)?;
    let cache_state = std::env::var("BIJUX_CI_CACHE_STATE")
        .unwrap_or_else(|_| "unspecified".to_string())
        .to_ascii_lowercase();
    let actual_os = std::env::consts::OS.to_string();
    let actual_architecture = std::env::consts::ARCH.to_string();
    let actual_rust_toolchain = rust_toolchain_version(workspace)?;
    let eligible_reference_observation = all_passed
        && source_worktree_clean
        && cache_state == "warm"
        && actual_os == contract.reference_runner.os
        && actual_architecture == contract.reference_runner.architecture
        && actual_rust_toolchain == contract.reference_runner.rust_toolchain;
    let history_path = workspace.path(&contract.timing_history);
    let mut history = load_fast_timing_history(&history_path)?;
    if eligible_reference_observation {
        history.observations.push(FastTimingObservation {
            observed_at_utc: chrono::Utc::now().to_rfc3339(),
            source_commit: source_commit.clone(),
            os: actual_os.clone(),
            architecture: actual_architecture.clone(),
            rust_toolchain: actual_rust_toolchain.clone(),
            cache_state: cache_state.clone(),
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
    let critical_path = critical_path.map(|(group, duration_milliseconds)| {
        serde_json::json!({
            "group": group,
            "duration_milliseconds": duration_milliseconds,
        })
    });
    write_json_pretty(
        &workspace.path(&contract.timing_report),
        &serde_json::json!({
            "schema_version": "bijux.ci.fast_timing.v1",
            "source_commit": source_commit,
            "contract_sha256": bijux_dna_infra::hash_file_sha256(&contract_path)?,
            "observed_at_utc": chrono::Utc::now().to_rfc3339(),
            "runner": {
                "os": actual_os,
                "architecture": actual_architecture,
                "rust_toolchain": actual_rust_toolchain,
                "cache_state": cache_state,
                "source_worktree_clean": source_worktree_clean,
            },
            "reference_runner": {
                "os": contract.reference_runner.os,
                "architecture": contract.reference_runner.architecture,
                "rust_toolchain": contract.reference_runner.rust_toolchain,
            },
            "warm_cache_target_milliseconds": contract.warm_cache_target_milliseconds,
            "eligible_reference_observation": eligible_reference_observation,
            "wall_time_milliseconds": wall_time_milliseconds,
            "all_passed": all_passed,
            "critical_path": critical_path,
            "groups": receipts,
            "warm_reference_history": {
                "sample_count": samples.len(),
                "p50_milliseconds": p50,
                "p95_milliseconds": p95,
                "p95_within_target": p95_within_target,
                "minimum_claim_sample_count": 5,
            },
        }),
    )?;
    Ok(OpsCommandOutcome { exit_code: i32::from(!all_passed), stdout, stderr })
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
    ensure_help_only("ci-test-all-sharded", args)?;
    let contract_path = workspace.path("configs/ci/full-shards.json");
    let contract: FullShardContract = serde_json::from_slice(&fs::read(&contract_path)?)
        .with_context(|| format!("parse {}", contract_path.display()))?;
    validate_full_shard_contract(&contract)?;

    let report_root = workspace.path(&contract.report_root);
    let build_authority_path = workspace.path(&contract.build_authority);
    let build_authority = read_ci_build_authority(&build_authority_path)?;
    require_ci_test_executor(workspace, &build_authority)?;
    let build_authority_sha256 = bijux_dna_infra::hash_file_sha256(&build_authority_path)?;
    bijux_dna_infra::ensure_dir(&report_root)?;
    let started = Instant::now();
    let shard_workspaces = contract
        .shards
        .iter()
        .map(|shard| materialize_full_shard_workspace(workspace, &report_root, shard))
        .collect::<Result<Vec<_>>>()?;
    let shard_roots = contract
        .shards
        .iter()
        .map(|shard| full_shard_roots(workspace, &contract, shard))
        .collect::<Vec<_>>();
    validate_full_shard_root_set(workspace, &shard_roots)?;
    let specs = contract
        .shards
        .iter()
        .zip(&shard_workspaces)
        .zip(&shard_roots)
        .map(|((shard, shard_workspace), roots)| {
            Ok(ParallelProcessSpec {
                label: shard.id.clone(),
                program: "cargo".to_string(),
                args: vec![
                    "nextest".to_string(),
                    "run".to_string(),
                    "--workspace".to_string(),
                    "--all-targets".to_string(),
                    "--all-features".to_string(),
                    "--locked".to_string(),
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
                ],
                envs: full_shard_env(roots)?,
                current_dir: Some(shard_workspace.clone()),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let outputs = ProcessRunner::new(workspace).run_parallel(specs)?;
    let mut all_passed = true;
    let mut shard_receipts = Vec::with_capacity(outputs.len());
    let mut stdout = String::new();
    let mut stderr = String::new();
    for (((shard, shard_workspace), roots), completed) in
        contract.shards.iter().zip(&shard_workspaces).zip(&shard_roots).zip(outputs)
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
            "environment_roots": full_shard_root_receipt(roots),
            "duration_milliseconds": completed.duration.as_millis(),
            "exit_code": exit_code,
            "log": workspace.rel(&log_path).to_string_lossy(),
        }));
    }
    write_json_pretty(
        &workspace.path("artifacts/ci/full-gate-timing.json"),
        &serde_json::json!({
            "schema_version": "bijux.ci.full_gate_timing.v2",
            "contract_sha256": bijux_dna_infra::hash_file_sha256(&contract_path)?,
            "build_authority": workspace.rel(&build_authority_path).to_string_lossy(),
            "build_authority_sha256": build_authority_sha256,
            "coverage": {
                "workspace": true,
                "all_targets": true,
                "all_features": true,
                "run_ignored": "all",
                "overlap_policy": "exactly_once",
                "workspace_isolation": contract.workspace_isolation,
                "output_isolation": "disjoint_per_shard_roots"
            },
            "wall_time_milliseconds": started.elapsed().as_millis(),
            "all_passed": all_passed,
            "shards": shard_receipts,
        }),
    )?;
    Ok(OpsCommandOutcome { exit_code: i32::from(!all_passed), stdout, stderr })
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

fn materialize_full_shard_workspace(
    workspace: &Workspace,
    report_root: &Path,
    shard: &FullShard,
) -> Result<PathBuf> {
    let shard_workspace = report_root.join("workspaces").join(&shard.id);
    if shard_workspace.exists() {
        std::fs::remove_dir_all(&shard_workspace).with_context(|| {
            format!("remove stale shard workspace {}", shard_workspace.display())
        })?;
    }
    let shard_parent = shard_workspace.parent().context("shard workspace has no parent")?;
    bijux_dna_infra::ensure_dir(shard_parent)?;
    let clone = ProcessRunner::new(workspace).run_owned(
        "git",
        &[
            "clone".to_string(),
            "--shared".to_string(),
            "--quiet".to_string(),
            workspace.root.display().to_string(),
            shard_workspace.display().to_string(),
        ],
    )?;
    if !clone.status.success() {
        anyhow::bail!(
            "clone shard workspace {}: {}",
            shard_workspace.display(),
            String::from_utf8_lossy(&clone.stderr)
        );
    }
    bijux_dna_infra::ensure_dir(shard_workspace.join("artifacts"))?;

    let tracked = ProcessRunner::new(workspace).run(&["git", "ls-files", "-z"])?;
    if !tracked.status.success() {
        anyhow::bail!(
            "list tracked files for shard workspace: {}",
            String::from_utf8_lossy(&tracked.stderr)
        );
    }
    for encoded in tracked.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let relative = std::str::from_utf8(encoded).context("tracked path is not UTF-8")?;
        let relative = Path::new(relative);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            anyhow::bail!("tracked path escapes shard workspace: {}", relative.display());
        }
        let source = workspace.root.join(relative);
        let destination = shard_workspace.join(relative);
        if let Some(parent) = destination.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
        std::fs::copy(&source, &destination).with_context(|| {
            format!("copy tracked shard source {} to {}", source.display(), destination.display())
        })?;
    }
    Ok(shard_workspace)
}

fn full_shard_roots(
    workspace: &Workspace,
    contract: &FullShardContract,
    shard: &FullShard,
) -> FullShardRoots {
    let artifact_root = workspace.path(&contract.shard_artifact_root).join(&shard.id);
    FullShardRoots {
        cargo_target: artifact_root.join("target/cargo"),
        test_target: artifact_root.join("target/test"),
        coverage_target: artifact_root.join("target/coverage"),
        test_tmp: artifact_root.join("tmp/test"),
        coverage_tmp: artifact_root.join("tmp/coverage"),
        test_profraw: artifact_root.join("coverage/profraw-test"),
        coverage_profraw: artifact_root.join("coverage/profraw-coverage"),
        artifact_root,
    }
}

fn validate_full_shard_root_set(
    workspace: &Workspace,
    shard_roots: &[FullShardRoots],
) -> Result<()> {
    let governed_artifacts = workspace.path("artifacts");
    for roots in shard_roots {
        for root in roots.all() {
            if !root.is_absolute() || !root.starts_with(&governed_artifacts) {
                anyhow::bail!("full shard output root is outside artifacts: {}", root.display());
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

fn full_shard_env(roots: &FullShardRoots) -> Result<Vec<(String, String)>> {
    for root in roots.all() {
        bijux_dna_infra::ensure_dir(root)?;
    }
    Ok(vec![
        ("ARTIFACT_ROOT".to_string(), roots.artifact_root.display().to_string()),
        ("ISO_ROOT".to_string(), roots.artifact_root.display().to_string()),
        ("CARGO_TARGET_DIR".to_string(), roots.cargo_target.display().to_string()),
        ("TEST_TARGET_DIR".to_string(), roots.test_target.display().to_string()),
        ("COV_TARGET_DIR".to_string(), roots.coverage_target.display().to_string()),
        ("TEST_TMP_DIR".to_string(), roots.test_tmp.display().to_string()),
        ("COV_TMP_DIR".to_string(), roots.coverage_tmp.display().to_string()),
        ("TEST_PROFRAW_DIR".to_string(), roots.test_profraw.display().to_string()),
        ("COV_PROFRAW_DIR".to_string(), roots.coverage_profraw.display().to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ])
}

fn full_shard_root_receipt(roots: &FullShardRoots) -> serde_json::Value {
    serde_json::json!({
        "artifact_root": roots.artifact_root.display().to_string(),
        "iso_root": roots.artifact_root.display().to_string(),
        "cargo_target_dir": roots.cargo_target.display().to_string(),
        "test_target_dir": roots.test_target.display().to_string(),
        "coverage_target_dir": roots.coverage_target.display().to_string(),
        "test_tmp_dir": roots.test_tmp.display().to_string(),
        "coverage_tmp_dir": roots.coverage_tmp.display().to_string(),
        "test_profraw_dir": roots.test_profraw.display().to_string(),
        "coverage_profraw_dir": roots.coverage_profraw.display().to_string(),
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

fn validate_full_shard_contract(contract: &FullShardContract) -> Result<()> {
    if contract.schema_version != FULL_SHARDS_SCHEMA_V2
        || !contract.coverage.workspace
        || !contract.coverage.all_targets
        || !contract.coverage.all_features
        || contract.coverage.run_ignored != "all"
        || contract.coverage.overlap_policy != "exactly_once"
        || contract.partition_strategy != "hash"
        || contract.workspace_isolation != "tracked_source_clone_with_disjoint_outputs"
        || contract.build_authority != "configs/ci/build-authority.json"
        || !(2..=16).contains(&contract.shards.len())
    {
        anyhow::bail!("full shard coverage contract is incomplete");
    }
    validate_artifact_relative_path(&contract.shard_artifact_root)?;
    validate_artifact_relative_path(&contract.report_root)?;
    let report_root = Path::new(&contract.report_root);
    let shard_artifact_root = Path::new(&contract.shard_artifact_root);
    if shard_artifact_root == report_root || !shard_artifact_root.starts_with(report_root) {
        anyhow::bail!("full shard artifact root must be a child of the report root");
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

fn validate_fast_group_contract(contract: &FastGroupContract) -> Result<()> {
    if contract.schema_version != FAST_GROUPS_SCHEMA_V1
        || !(2..=16).contains(&contract.max_parallelism)
        || !(2..=16).contains(&contract.groups.len())
        || !(5..=100).contains(&contract.history_limit)
        || contract.warm_cache_target_milliseconds == 0
        || contract.reference_runner.os.is_empty()
        || contract.reference_runner.architecture.is_empty()
        || contract.reference_runner.rust_toolchain.is_empty()
    {
        anyhow::bail!("ci-fast group contract is incomplete");
    }
    validate_artifact_relative_path(&contract.report_root)?;
    validate_artifact_relative_path(&contract.timing_report)?;
    validate_artifact_relative_path(&contract.timing_history)?;

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
            schema_version: FAST_TIMING_HISTORY_SCHEMA_V1.to_string(),
            observations: Vec::new(),
        });
    }
    let history: FastTimingHistory = serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse {}", path.display()))?;
    if history.schema_version != FAST_TIMING_HISTORY_SCHEMA_V1 {
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
                && observation.cache_state == "warm"
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
    use super::{
        full_shard_env, full_shard_root_receipt, full_shard_roots,
        materialize_full_shard_workspace, percentile, read_ci_build_authority,
        reference_timing_samples, validate_ci_test_executor_observation,
        validate_fast_group_contract, validate_full_shard_contract, validate_full_shard_root_set,
        FastGroup, FastGroupContract, FastReferenceRunner, FastTimingHistory,
        FastTimingObservation, FullShard, FullShardContract, FullShardCoverage,
        FAST_TIMING_HISTORY_SCHEMA_V1,
    };
    use crate::runtime::workspace::Workspace;

    fn contract() -> FullShardContract {
        FullShardContract {
            schema_version: "bijux.ci.full_shards.v2".to_string(),
            coverage: FullShardCoverage {
                workspace: true,
                all_targets: true,
                all_features: true,
                run_ignored: "all".to_string(),
                overlap_policy: "exactly_once".to_string(),
            },
            partition_strategy: "hash".to_string(),
            workspace_isolation: "tracked_source_clone_with_disjoint_outputs".to_string(),
            build_authority: "configs/ci/build-authority.json".to_string(),
            shard_artifact_root: "artifacts/ci/full-shards/shards".to_string(),
            report_root: "artifacts/ci/full-shards".to_string(),
            shards: (1..=4)
                .map(|index| FullShard {
                    id: format!("hash-{index}-of-4"),
                    partition: format!("hash:{index}/4"),
                })
                .collect(),
        }
    }

    fn fast_contract() -> FastGroupContract {
        FastGroupContract {
            schema_version: "bijux.ci.fast_groups.v1".to_string(),
            max_parallelism: 2,
            warm_cache_target_milliseconds: 300_000,
            history_limit: 40,
            report_root: "artifacts/ci/ci-fast-groups".to_string(),
            timing_report: "artifacts/ci/ci-fast-timing.json".to_string(),
            timing_history: "artifacts/ci/ci-fast-timing-history.json".to_string(),
            reference_runner: FastReferenceRunner {
                os: "linux".to_string(),
                architecture: "x86_64".to_string(),
                rust_toolchain: "1.95.0".to_string(),
            },
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
        overlap.shards[3].partition = "hash:3/4".to_string();
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

        let mut unknown_authority = contract();
        unknown_authority.build_authority = "configs/ci/another-authority.json".to_string();
        assert!(validate_full_shard_contract(&unknown_authority).is_err());
    }

    #[test]
    fn full_shard_runtime_roots_are_absolute_complete_and_disjoint() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let contract = contract();
        let roots = contract
            .shards
            .iter()
            .map(|shard| full_shard_roots(&workspace, &contract, shard))
            .collect::<Vec<_>>();
        validate_full_shard_root_set(&workspace, &roots)?;

        let first_env =
            full_shard_env(&roots[0])?.into_iter().collect::<std::collections::BTreeMap<_, _>>();
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
            assert!(path.starts_with(workspace.path("artifacts")));
        }
        assert_ne!(first_env["CARGO_TARGET_DIR"], first_env["TEST_TARGET_DIR"]);
        assert_ne!(first_env["TEST_TARGET_DIR"], first_env["COV_TARGET_DIR"]);
        assert_ne!(first_env["TEST_TMP_DIR"], first_env["COV_TMP_DIR"]);
        assert_ne!(first_env["TEST_PROFRAW_DIR"], first_env["COV_PROFRAW_DIR"]);
        let receipt = full_shard_root_receipt(&roots[0]);
        assert_eq!(receipt["artifact_root"], first_env["ARTIFACT_ROOT"]);
        assert_eq!(receipt["iso_root"], first_env["ISO_ROOT"]);
        assert_eq!(receipt["cargo_target_dir"], first_env["CARGO_TARGET_DIR"]);
        assert_eq!(receipt["test_target_dir"], first_env["TEST_TARGET_DIR"]);
        assert_eq!(receipt["coverage_target_dir"], first_env["COV_TARGET_DIR"]);
        assert_eq!(receipt["test_tmp_dir"], first_env["TEST_TMP_DIR"]);
        assert_eq!(receipt["coverage_tmp_dir"], first_env["COV_TMP_DIR"]);
        assert_eq!(receipt["test_profraw_dir"], first_env["TEST_PROFRAW_DIR"]);
        assert_eq!(receipt["coverage_profraw_dir"], first_env["COV_PROFRAW_DIR"]);
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
            FullShard { id: "isolation-proof".to_string(), partition: "hash:1/4".to_string() };

        let isolated = materialize_full_shard_workspace(&workspace, report_root.path(), &shard)?;
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
    }

    #[test]
    fn timing_percentiles_use_nearest_rank() {
        assert_eq!(percentile(&[], 95), None);
        assert_eq!(percentile(&[100, 20, 40, 60, 80], 50), Some(60));
        assert_eq!(percentile(&[100, 20, 40, 60, 80], 95), Some(100));
    }

    #[test]
    fn timing_samples_require_exact_warm_reference_runner() {
        let reference = fast_contract().reference_runner;
        let observation =
            |os: &str, architecture: &str, rust_toolchain: &str, cache_state: &str| {
                FastTimingObservation {
                    observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
                    source_commit: "0123456789abcdef".to_string(),
                    os: os.to_string(),
                    architecture: architecture.to_string(),
                    rust_toolchain: rust_toolchain.to_string(),
                    cache_state: cache_state.to_string(),
                    wall_time_milliseconds: 42,
                }
            };
        let history = FastTimingHistory {
            schema_version: FAST_TIMING_HISTORY_SCHEMA_V1.to_string(),
            observations: vec![
                observation("linux", "x86_64", "1.95.0", "warm"),
                observation("macos", "aarch64", "1.95.0", "warm"),
                observation("linux", "x86_64", "1.94.0", "warm"),
                observation("linux", "x86_64", "1.95.0", "cold"),
            ],
        };

        assert_eq!(reference_timing_samples(&history, &reference), vec![42]);
    }
}
