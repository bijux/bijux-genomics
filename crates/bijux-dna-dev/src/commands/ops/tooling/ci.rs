use super::super::suite_selection::governed_slow_expression;
use std::collections::BTreeSet;
use std::path::{Component, Path};
use std::time::Instant;

use serde::Deserialize;

use super::{
    artifact_env, artifact_root_path, ci_test_env, ensure_help_only, env_or_default, fs,
    merge_outcomes, read_coverage_runner_flag, resolved_nextest_expression,
    resolved_nextest_profile, resolved_nextest_threads, resolved_run_ignored, run_check_ids,
    run_make_target, run_program_with_env, set_assets_readonly, write_json_pretty, Context,
    OpsCommandOutcome, Result, Workspace,
};
use crate::runtime::process::{ParallelProcessSpec, ProcessRunner};

const FULL_SHARDS_SCHEMA_V1: &str = "bijux.ci.full_shards.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FullShardContract {
    schema_version: String,
    coverage: FullShardCoverage,
    partition_strategy: String,
    shared_target_root: String,
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
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "install".to_string(),
            "--locked".to_string(),
            "cargo-nextest".to_string(),
            "cargo-llvm-cov".to_string(),
            "cargo-deny".to_string(),
        ],
        &artifact_env(workspace)?,
    )
}

pub(in super::super) fn tooling_ci_fast(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("ci-fast", args)?;
    run_make_target(workspace, "_ci-fast")
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

    let target_root = workspace.path(&contract.shared_target_root);
    let report_root = workspace.path(&contract.report_root);
    bijux_dna_infra::ensure_dir(&target_root)?;
    bijux_dna_infra::ensure_dir(&report_root)?;
    let mut envs = ci_test_env(workspace, false)?;
    envs.push(("CARGO_TARGET_DIR".to_string(), target_root.display().to_string()));
    let started = Instant::now();
    let specs = contract
        .shards
        .iter()
        .map(|shard| ParallelProcessSpec {
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
            envs: envs.clone(),
        })
        .collect::<Vec<_>>();
    let outputs = ProcessRunner::new(workspace).run_parallel(specs)?;
    let mut all_passed = true;
    let mut shard_receipts = Vec::with_capacity(outputs.len());
    let mut stdout = String::new();
    let mut stderr = String::new();
    for (shard, completed) in contract.shards.iter().zip(outputs) {
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
            "duration_milliseconds": completed.duration.as_millis(),
            "exit_code": exit_code,
            "log": workspace.rel(&log_path).to_string_lossy(),
        }));
    }
    write_json_pretty(
        &workspace.path("artifacts/ci/full-gate-timing.json"),
        &serde_json::json!({
            "schema_version": "bijux.ci.full_gate_timing.v1",
            "contract_sha256": bijux_dna_infra::hash_file_sha256(&contract_path)?,
            "coverage": {
                "workspace": true,
                "all_targets": true,
                "all_features": true,
                "run_ignored": "all",
                "overlap_policy": "exactly_once"
            },
            "wall_time_milliseconds": started.elapsed().as_millis(),
            "all_passed": all_passed,
            "shards": shard_receipts,
        }),
    )?;
    Ok(OpsCommandOutcome { exit_code: i32::from(!all_passed), stdout, stderr })
}

fn validate_full_shard_contract(contract: &FullShardContract) -> Result<()> {
    if contract.schema_version != FULL_SHARDS_SCHEMA_V1
        || !contract.coverage.workspace
        || !contract.coverage.all_targets
        || !contract.coverage.all_features
        || contract.coverage.run_ignored != "all"
        || contract.coverage.overlap_policy != "exactly_once"
        || contract.partition_strategy != "hash"
        || !(2..=16).contains(&contract.shards.len())
    {
        anyhow::bail!("full shard coverage contract is incomplete");
    }
    validate_artifact_relative_path(&contract.shared_target_root)?;
    validate_artifact_relative_path(&contract.report_root)?;
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
    use super::{validate_full_shard_contract, FullShard, FullShardContract, FullShardCoverage};

    fn contract() -> FullShardContract {
        FullShardContract {
            schema_version: "bijux.ci.full_shards.v1".to_string(),
            coverage: FullShardCoverage {
                workspace: true,
                all_targets: true,
                all_features: true,
                run_ignored: "all".to_string(),
                overlap_policy: "exactly_once".to_string(),
            },
            partition_strategy: "hash".to_string(),
            shared_target_root: "artifacts/target".to_string(),
            report_root: "artifacts/ci/full-shards".to_string(),
            shards: (1..=4)
                .map(|index| FullShard {
                    id: format!("hash-{index}-of-4"),
                    partition: format!("hash:{index}/4"),
                })
                .collect(),
        }
    }

    #[test]
    fn full_shard_contract_covers_hash_space_exactly_once() {
        assert!(validate_full_shard_contract(&contract()).is_ok());
    }

    #[test]
    fn full_shard_contract_rejects_overlap_and_escaping_outputs() {
        let mut overlap = contract();
        overlap.shards[3].partition = "hash:3/4".to_string();
        assert!(validate_full_shard_contract(&overlap).is_err());

        let mut escaping = contract();
        escaping.report_root = "../reports".to_string();
        assert!(validate_full_shard_contract(&escaping).is_err());
    }
}
