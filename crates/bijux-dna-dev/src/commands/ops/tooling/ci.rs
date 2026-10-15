//! Registered CI command facade and shared artifact admission.
mod complete;
mod fast;
mod full_execution;

pub(in super::super) use complete::tooling_ci_test_complete;
pub(in super::super) use fast::tooling_ci_fast;

use crate::application::ci::admit_compiled_executable_paths;
use crate::application::ci::CompiledWorkspaceExecutables;
use crate::application::ci::CoverageAttestation;
use crate::application::ci::ExpectedTestInventory;
use crate::application::ci::NextestReuseInputs;
use crate::application::ci::ShardDiscoveryPlan;
use crate::commands::ops::suite_selection::governed_slow_expression;
use crate::commands::ops::tooling::artifact_env;
use crate::commands::ops::tooling::artifact_root_path;
use crate::commands::ops::tooling::ci_test_env;
use crate::commands::ops::tooling::ensure_help_only;
use crate::commands::ops::tooling::env_or_default;
use crate::commands::ops::tooling::merge_outcomes;
use crate::commands::ops::tooling::read_coverage_runner_flag;
use crate::commands::ops::tooling::resolved_nextest_expression;
use crate::commands::ops::tooling::resolved_nextest_profile;
use crate::commands::ops::tooling::resolved_nextest_threads;
use crate::commands::ops::tooling::resolved_run_ignored;
use crate::commands::ops::tooling::run_check_ids;
use crate::commands::ops::tooling::run_make_target;
use crate::commands::ops::tooling::run_program_with_env;
use crate::commands::ops::tooling::set_assets_readonly;
use crate::commands::ops::tooling::write_json_pretty;
use crate::model::ops::OpsCommandOutcome;
use crate::runtime::compilation_cache::CompilationCache;
use crate::runtime::compiled_products::discover_native_inventory;
use crate::runtime::compiled_products::ExecutableSeal;
use crate::runtime::compiled_products::LocalCompiledProducts;
use crate::runtime::owned_artifacts::owned_children;
use crate::runtime::owned_artifacts::remove_empty_owned_directory;
use crate::runtime::owned_artifacts::remove_owned_tree as remove_full_shard_ephemera;
use crate::runtime::owned_artifacts::remove_owned_trees;
use crate::runtime::process::ParallelProcessSpec;
use crate::runtime::process::ProcessRunner;
use crate::runtime::source_snapshot::IsolatedSourceIdentity;
use crate::runtime::source_snapshot::TrackedSourceSnapshot;
use crate::runtime::workspace::Workspace;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_policies::validate_ci_build_authority;
use bijux_dna_policies::CiBuildAuthority;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

const FULL_SHARDS_SCHEMA_V5: &str = "bijux.ci.full_shards.v5";

const FULL_SHARD_SCRATCH_ROOT_ENV: &str = "BIJUX_CI_FULL_SHARD_SCRATCH_ROOT";

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

struct ArchivedExecutableSeal {
    extracted: ExecutableSeal,
    producer: ExecutableSeal,
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
        let mut paths = self.shard_artifact_roots.clone();
        paths.push(self.archive_root.clone());
        let children = owned_children(&self.scratch_workspaces);
        let mut failures = Vec::new();
        match children {
            Ok(children) => paths.extend(children),
            Err(error) => failures.push(format!("{error:#}")),
        }
        if let Err(error) = remove_owned_trees(&paths) {
            failures.push(format!("{error:#}"));
        }
        if let Err(error) = remove_empty_owned_directory(&self.scratch_workspaces) {
            failures.push(format!("{error:#}"));
        }
        anyhow::ensure!(failures.is_empty(), "full shard cleanup failed: {}", failures.join("; "));
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
    let started = Instant::now();
    let (local_products, args) = match args.split_first() {
        Some((flag, remaining)) if flag == "--local-products" => (true, remaining),
        _ => (false, args),
    };
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
    let source_snapshot = TrackedSourceSnapshot::capture(workspace)?;
    let source_identity = source_snapshot.commit();
    let source_patch_sha256 = source_snapshot.patch_sha256();
    let source_clean = source_worktree_clean(workspace)?;
    let receipt_path = ci_output_path(workspace, "artifacts/ci/full-gate-timing.json")?;
    let mut session: (Option<CompilationCache>, Option<FullShardEphemera>) = (None, None);
    crate::runtime::gate_receipt::run_reported(
        &receipt_path,
        serde_json::json!({
            "schema_version": "bijux.ci.full_gate_timing.v4",
            "execution_resources": {"state": "pending_admission"},
            "archive_build_jobs": contract.archive_build_jobs,
            "state": "running",
            "source_commit": source_identity,
            "source_worktree_clean": source_clean,
            "source_patch_sha256": source_patch_sha256,
            "all_passed": false,
        }),
        started,
        &mut session,
        |session, receipt| {
            let expected_source = IsolatedSourceIdentity::capture(workspace)?;
            expected_source.require_snapshot(&source_snapshot)?;
            receipt["isolated_source_condition"] = expected_source.descriptor();
            let execution_budget = full_execution::admit_budget(receipt, || {
                full_execution::ExecutionBudget::resolve(selected_shards.len())
            })?;
            full_execution::admit_native_transport(receipt, &execution_budget, local_products)?;
            let native_worker_admission = receipt["native_worker_admission"].clone();
            let archive_build_jobs = full_execution::resolve_compiler_jobs(
                receipt,
                contract.archive_build_jobs,
                execution_budget.worker_budget,
                local_products,
            )?;
            let compiler_admission = receipt["compiler_admission"].clone();
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
            let compilation_cache =
                session.0.insert(CompilationCache::acquire(&storage.scratch_root)?);
            anyhow::ensure!(
                compilation_cache.target() == archive_roots.cargo_target,
                "archive target differs from its exclusive cache owner"
            );
            session.1 =
                Some(FullShardEphemera::new(&storage.scratch_root, &archive_roots, &shard_roots));
            prepare_full_archive_roots(&archive_roots)?;
            let preparation_started = Instant::now();
            let archive_workspace = if source_snapshot.is_clean() {
                let path = storage.scratch_root.join("compile-cache/workspace");
                source_snapshot.materialize_cached_clean(&path)?;
                path
            } else {
                materialize_full_shard_workspace(
                    &source_snapshot,
                    &storage.scratch_root,
                    "archive-build",
                )?
            };
            let compiled_source = Workspace { root: archive_workspace.clone() };
            receipt["compiled_source_before_production"] =
                expected_source.verify(&compiled_source)?;
            let archive_preparation = preparation_started.elapsed();
            let producer_result = (|| -> Result<_> {
                let local_compiled = if local_products {
                    Some(LocalCompiledProducts::build(
                        workspace,
                        &archive_workspace,
                        compilation_cache.target(),
                        &full_archive_env(&archive_roots, archive_build_jobs)?,
                        &report_root,
                    )?)
                } else {
                    None
                };
                let archive_output = if local_compiled.is_none() {
                    let archive_spec = ParallelProcessSpec {
                        label: "archive-build".to_string(),
                        program: "cargo".to_string(),
                        args: full_archive_args(&archive_roots.archive_file),
                        envs: full_archive_env(&archive_roots, archive_build_jobs)?,
                        current_dir: Some(archive_workspace.clone()),
                    };
                    let mut archive_outputs = ProcessRunner::new(workspace)
                        .run_parallel_logged(vec![archive_spec], &report_root)?;
                    Some(archive_outputs.pop().context("archive build was not collected")?)
                } else {
                    None
                };
                Ok((local_compiled, archive_output))
            })();
            let producer_source_postcondition = expected_source.postcondition(&compiled_source);
            receipt["compiled_source_after_production"] = producer_source_postcondition.clone();
            receipt["product_preparation_error"] =
                serde_json::json!(producer_result.as_ref().err().map(|error| format!("{error:#}")));
            let (mut local_compiled, archive_output) = producer_result?;
            anyhow::ensure!(
                producer_source_postcondition["unchanged"] == true,
                "compiler source changed during product preparation"
            );
            if let Some(output) = &archive_output {
                let mut log = output.output.stdout.clone();
                log.extend_from_slice(&output.output.stderr);
                bijux_dna_infra::write_bytes(&archive_roots.build_log, &log)?;
            }
            let archive_exit_code = archive_output
                .as_ref()
                .map_or(0, |output| output.output.status.code().unwrap_or(1));
            let archive_passed = local_compiled.is_some()
                || (archive_exit_code == 0 && archive_roots.archive_file.is_file());
            let archive_sha256 = (archive_output.is_some() && archive_passed)
                .then(|| bijux_dna_infra::hash_file_sha256(&archive_roots.archive_file))
                .transpose()?;
            if archive_sha256.is_some() {
                set_full_archive_readonly(&archive_roots.archive_file)?;
            }
            let mut archive_receipt = archive_output.as_ref().map(|output| serde_json::json!({
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
        "build_jobs": archive_build_jobs,
        "duration_milliseconds": output.duration.as_millis(),
        "exit_code": archive_exit_code,
        "log": workspace.rel(&archive_roots.build_log).to_string_lossy(),
        "stdout_log": workspace.rel(&report_root.join("archive-build.stdout.log")).to_string_lossy(),
        "stderr_log": workspace.rel(&report_root.join("archive-build.stderr.log")).to_string_lossy(),
        "read_only_during_shard_execution": archive_passed,
    })).unwrap_or(serde_json::Value::Null);
            if archive_output.is_some() {
                write_json_pretty(
                    &report_root.join("archive-build.receipt.json"),
                    &archive_receipt,
                )?;
            }
            let mut stdout = archive_output
                .as_ref()
                .map(|output| String::from_utf8_lossy(&output.output.stdout).into_owned())
                .unwrap_or_default();
            let mut stderr = archive_output
                .as_ref()
                .map(|output| String::from_utf8_lossy(&output.output.stderr).into_owned())
                .unwrap_or_default();
            if !archive_passed {
                *receipt = serde_json::json!({
                    "schema_version": "bijux.ci.full_gate_timing.v4",
                    "execution_resources": execution_budget,
                    "native_worker_admission": native_worker_admission,
                    "compiler_admission": compiler_admission,
                    "archive_build_jobs": archive_build_jobs,
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
                    "isolated_source_condition": expected_source.descriptor(),
                    "compiled_source_after_production": producer_source_postcondition,
                    "shards": [],
                });
                return Ok(OpsCommandOutcome { exit_code: 1, stdout, stderr });
            }
            let source_seal = TrackedSourceSnapshot::capture(&compiled_source)?;
            anyhow::ensure!(
                source_seal.commit() == source_identity
                    && source_seal.patch_sha256() == source_patch_sha256,
                "compiled source changed from its admitted source snapshot"
            );
            receipt["compiled_source"] = serde_json::json!({
                "workspace": archive_workspace, "source_commit": source_seal.commit(),
                "source_patch_sha256": source_seal.patch_sha256(),
            });
            receipt["compiled_products"] =
                serde_json::to_value(local_compiled.as_ref().map(|products| &products.receipt))?;
            let mut archive_seal = None;
            let (reuse_inputs, compiled_target) = if local_compiled.is_some() {
                (None, compilation_cache.target().to_path_buf())
            } else {
                prepare_full_archive_extraction(&archive_roots.archive_extract)?;
                let extraction_args = full_archive_extraction_args(&archive_roots);
                let mut extraction_outputs = ProcessRunner::new(workspace).run_parallel_logged(
                    vec![ParallelProcessSpec {
                        label: "archive-extract".to_string(),
                        program: "cargo".to_string(),
                        args: extraction_args.clone(),
                        envs: full_archive_env(&archive_roots, archive_build_jobs)?,
                        current_dir: Some(archive_workspace.clone()),
                    }],
                    &report_root,
                )?;
                let extraction =
                    extraction_outputs.pop().context("archive extraction was not collected")?;
                archive_receipt["extraction"] = serde_json::json!({
                    "command": extraction_args, "duration_milliseconds": extraction.duration.as_millis(),
                    "exit_code": extraction.output.status.code().unwrap_or(1),
                    "stdout_log": report_root.join("archive-extract.stdout.log"),
                    "stderr_log": report_root.join("archive-extract.stderr.log"),
                    "native_test_discovery": false,
                });
                receipt["archive"] = archive_receipt.clone();
                anyhow::ensure!(
                    extraction.output.status.success(),
                    "archive extraction failed; see archive-extract.stderr.log"
                );
                seal_full_archive_extraction(&archive_roots.archive_extract)?;
                let mut seal = capture_archive_executable_seal(&archive_roots, &archive_workspace)?;
                seal.extracted.require_readonly()?;
                archive_receipt["input_manifest"] = seal
                    .extracted
                    .publish_manifest(&report_root.join("compiled-inputs.manifest.json"))?;
                archive_receipt["producer_input_manifest"] = seal.producer.publish_manifest(
                    &report_root.join("compiled-producer-inputs.manifest.json"),
                )?;
                receipt["archive"] = archive_receipt.clone();
                archive_seal = Some(seal);
                let target = archive_roots.archive_extract.join("target");
                let reuse = archive_reuse_inputs(&archive_roots.archive_extract)?;
                (Some(reuse), target)
            };
            let reuse_inputs = local_compiled
                .as_ref()
                .map(|products| &products.reuse)
                .or(reuse_inputs.as_ref())
                .context("compiled test reuse inputs are absent")?;
            receipt["compiled_source_before_discovery"] =
                expected_source.verify(&compiled_source)?;
            let (inventory_bytes, inventory_duration) = if let Some(products) = &local_compiled {
                products.discover_inventory(
                    workspace,
                    &archive_workspace,
                    &full_archive_env(&archive_roots, archive_build_jobs)?,
                    &report_root,
                    execution_budget.worker_budget,
                )?
            } else {
                discover_native_inventory(
                    workspace,
                    &archive_workspace,
                    reuse_inputs,
                    &full_archive_env(&archive_roots, archive_build_jobs)?,
                    &report_root,
                    execution_budget.worker_budget,
                )?
            };
            receipt["compiled_source_after_discovery"] =
                expected_source.verify(&compiled_source)?;
            let expected_inventory = ExpectedTestInventory::parse(&inventory_bytes)?;
            let executables = CompiledWorkspaceExecutables::parse(&inventory_bytes)?;
            let inventory_log = report_root.join("test-inventory.stdout.log");
            let inventory_receipt = serde_json::json!({
                "schema_version": "bijux.ci.full_test_inventory.v1",
                "path": workspace.rel(&inventory_log).to_string_lossy(),
                "sha256": bijux_dna_infra::hash_file_sha256(&inventory_log)?,
                "workspace_executables": &executables,
                "expected_case_count": expected_inventory.case_count(),
                "expected_ignored_case_count": expected_inventory.ignored_case_count(),
                "duration_milliseconds": inventory_duration.as_millis(),
                "discovery_mode": "bounded_native_binary_subsets",
                "native_worker_budget": execution_budget.worker_budget,
                "source_commit": source_identity,
                "source_patch_sha256": source_patch_sha256,
                "archive_sha256": archive_sha256,
            });
            write_json_pretty(
                &report_root.join("test-inventory.receipt.json"),
                &inventory_receipt,
            )?;
            let shard_plans = selected_shards
                .iter()
                .map(|shard| {
                    let plan = ShardDiscoveryPlan::from_inventory(
                        &expected_inventory,
                        &shard.partition,
                        &build_authority.test_executor.version,
                    )?;
                    Ok(plan)
                })
                .collect::<Result<Vec<_>>>()?;
            let crate_gate_inputs =
                crate::runtime::crate_gate_inputs::CrateGateProofInputs::capture(
                    &inventory_log,
                    &archive_workspace,
                    &compiled_target,
                    crate::runtime::crate_gate_inputs::CrateGateProducerIdentity::new(
                        source_identity,
                        &source_patch_sha256,
                    )?,
                )?;
            let full_execution::ShardExecution {
                workspaces: shard_workspaces,
                outputs,
                preparation_milliseconds: shard_preparation,
                crate_gate_proof_inputs: shard_proof_inputs,
            } = full_execution::run_shards(
                full_execution::ShardInputs {
                    workspace,
                    shards: &selected_shards,
                    roots: &shard_roots,
                    plans: &shard_plans,
                    inventory: &expected_inventory,
                    inventory_bytes: &inventory_bytes,
                    snapshot: &source_snapshot,
                    scratch_root: &storage.scratch_root,
                    report_root: &report_root,
                    reuse: reuse_inputs,
                    executables: &executables,
                    compiled_target: &compiled_target,
                    crate_gate_inputs: &crate_gate_inputs,
                },
                &execution_budget,
            )?;
            let coverage = CoverageAttestation::reconcile_terminals(
                &expected_inventory,
                outputs
                    .iter()
                    .zip(&shard_plans)
                    .map(|(completed, plan)| (completed.label(), completed.events(), plan)),
            );
            let mut all_passed = coverage.selected_shards_valid
                && (selected_shards.len() != contract.shards.len() || coverage.complete);
            let mut shard_receipts = Vec::with_capacity(outputs.len());
            for (((shard, shard_workspace), roots), completed) in
                selected_shards.iter().zip(&shard_workspaces).zip(&shard_roots).zip(&outputs)
            {
                let log_path = report_root.join(format!("{}.log", shard.id));
                let mut log = completed.stdout().to_vec();
                log.extend_from_slice(completed.stderr());
                bijux_dna_infra::write_bytes(&log_path, &log)?;
                let exit_code = completed.exit_code();
                all_passed &= completed.succeeded();
                stdout.push_str(&String::from_utf8_lossy(completed.stdout()));
                stderr.push_str(&String::from_utf8_lossy(completed.stderr()));
                let mut environment_roots =
                    full_shard_root_receipt(roots, &archive_roots, shard_workspace);
                environment_roots["compiled_target_dir"] = serde_json::json!(compiled_target);
                if local_products {
                    environment_roots["archive_extract_dir"] = serde_json::Value::Null;
                    environment_roots["snapshot_source_authority"] =
                        serde_json::json!("local_compile_time_metadata");
                }
                shard_receipts.push(serde_json::json!({
            "id": completed.label(),
            "state": if completed.is_empty() { "expected_empty" } else { "executed" },
            "native_execution_observed": !completed.is_empty(),
            "partition": shard.partition,
            "workspace": (!completed.is_empty()).then(|| workspace.rel(shard_workspace).to_string_lossy()),
            "environment_roots": environment_roots,
            "duration_milliseconds": completed.duration().as_millis(),
            "duration_scope": "sum_of_terminally_sequenced_native_chunk_preparation_and_execution",
            "preparation_milliseconds": shard_preparation.get(&shard.id),
            "crate_gate_proof_inputs": shard_proof_inputs.get(&shard.id),
            "exit_code": exit_code,
            "log": workspace.rel(&log_path).to_string_lossy(),
            "stdout_log": workspace.rel(&report_root.join(format!("{}.stdout.log", shard.id))).to_string_lossy(),
            "stderr_log": workspace.rel(&report_root.join(format!("{}.stderr.log", shard.id))).to_string_lossy(),
        }));
            }
            if let Some(products) = &mut local_compiled {
                all_passed &= products.verify_unchanged();
                receipt["compiled_products"] = products.receipt.clone();
                write_json_pretty(
                    &report_root.join("compiled-products.receipt.json"),
                    &products.receipt,
                )?;
            } else {
                let archive_sha256_after =
                    bijux_dna_infra::hash_file_sha256(&archive_roots.archive_file)?;
                let archive_checksum_stable =
                    archive_sha256.as_deref() == Some(archive_sha256_after.as_str());
                all_passed &= archive_checksum_stable;
                archive_receipt["artifact_sha256_after_execution"] =
                    serde_json::Value::String(archive_sha256_after);
                archive_receipt["checksum_stable"] =
                    serde_json::Value::Bool(archive_checksum_stable);
                let seal = archive_seal.as_ref().context("archive executable seal is absent")?;
                let observed = seal.extracted.verify();
                all_passed &= observed.unchanged;
                archive_receipt["input_verification"] = observed.receipt;
                let producer = seal.producer.verify();
                all_passed &= producer.unchanged;
                archive_receipt["producer_input_verification"] = producer.receipt;
                receipt["archive"] = archive_receipt.clone();
            }
            let mut source_postconditions = vec![expected_source.postcondition(&compiled_source)];
            source_postconditions.extend(
                shard_workspaces
                    .iter()
                    .zip(&outputs)
                    .filter(|(_, terminal)| !terminal.is_empty())
                    .map(|(root, _)| {
                        expected_source.postcondition(&Workspace { root: root.clone() })
                    }),
            );
            let source_unchanged =
                source_postconditions.iter().all(|observed| observed["unchanged"] == true);
            all_passed &= source_unchanged;
            *receipt = serde_json::json!({
                "schema_version": "bijux.ci.full_gate_timing.v4",
                "execution_resources": execution_budget,
                    "native_worker_admission": native_worker_admission,
                    "compiler_admission": compiler_admission,
                "archive_build_jobs": archive_build_jobs,
                "state": "complete",
                "source_commit": source_identity,
                "source_worktree_clean": source_clean,
                "source_patch_sha256": source_patch_sha256,
                "isolated_source_condition": expected_source.descriptor(),
                "isolated_source_postconditions": source_postconditions,
                "compiled_source_verification": {
                    "workspace": archive_workspace,
                    "source_commit_after_execution": source_postconditions[0]["observed"]["sha"],
                    "source_patch_sha256_after_execution": source_postconditions[0]["observed"]["patch_sha256"],
                    "unchanged": source_postconditions[0]["unchanged"],
                    "observation": source_postconditions[0],
                },
                "contract_sha256": contract_sha256,
                "build_authority": workspace.rel(&build_authority_path).to_string_lossy(),
                "build_authority_sha256": build_authority_sha256,
                "runtime_dependencies": runtime_dependencies,
                "archive": archive_receipt,
                "compiled_products": local_compiled.as_ref().map(|products| &products.receipt),
                "product_transport": if local_products { "local_metadata" } else { "archive" },
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
                    "shards_milliseconds": shard_preparation.values().map(|&value| u128::from(value)).sum::<u128>(),
                    "shards_measurement": "sum_of_overlapping_slot_preparation_durations",
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
                    "output_isolation": if local_products { "exclusive_compilation_cache_with_disjoint_workspaces_and_runtime_roots" } else { "shared_read_only_extraction_with_disjoint_workspaces_and_runtime_roots" }
                },
                "wall_time_milliseconds": started.elapsed().as_millis(),
                "all_passed": all_passed,
                "shards": shard_receipts,
            });
            Ok(OpsCommandOutcome { exit_code: i32::from(!all_passed), stdout, stderr })
        },
        |session, receipt| {
            if let Some(ephemera) = &mut session.1 {
                observe_full_shard_cleanup(receipt, ephemera)
            } else {
                receipt["cleanup"] = serde_json::json!({"completed":true,"required":false});
                Ok(())
            }
        },
    )
}

fn observe_full_shard_cleanup(
    receipt: &mut serde_json::Value,
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
    cleanup
}

#[cfg(test)]
fn finish_full_shard_receipt(
    path: &Path,
    receipt: serde_json::Value,
    started: Instant,
    ephemera: &mut FullShardEphemera,
) -> Result<()> {
    crate::runtime::gate_receipt::run_reported(
        path,
        receipt,
        started,
        ephemera,
        |_, _| Ok(()),
        |ephemera, receipt| observe_full_shard_cleanup(receipt, ephemera),
    )
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

fn full_archive_extraction_args(archive: &FullArchiveRoots) -> Vec<String> {
    let mut args = full_inventory_args(archive);
    args.extend(["--list-type".to_string(), "binaries-only".to_string()]);
    args
}

fn full_shard_args(
    shard: &FullShard,
    reuse: &NextestReuseInputs,
    workspace_remap: &Path,
    test_threads: usize,
) -> Result<Vec<String>> {
    anyhow::ensure!(test_threads > 0, "full-shard case threads must be positive");
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
        "--test-threads".to_string(),
        test_threads.to_string(),
    ];
    args.extend(reuse.arguments(workspace_remap)?);
    Ok(args)
}

fn archive_reuse_inputs(archive_extract: &Path) -> Result<NextestReuseInputs> {
    let target = archive_extract.join("target");
    NextestReuseInputs::new(
        target.join("nextest/cargo-metadata.json"),
        target.join("nextest/binaries-metadata.json"),
        target,
    )
}

fn capture_archive_executable_seal(
    archive: &FullArchiveRoots,
    source: &Path,
) -> Result<ArchivedExecutableSeal> {
    let extracted_target = archive.archive_extract.join("target");
    let cargo_metadata = extracted_target.join("nextest/cargo-metadata.json");
    let binary_metadata = extracted_target.join("nextest/binaries-metadata.json");
    let paths = admit_compiled_executable_paths(
        &fs::read(&cargo_metadata)?,
        &fs::read(&binary_metadata)?,
        source,
        &archive.cargo_target,
    )?;
    let producer = ExecutableSeal::capture(&paths)?;
    let original = producer.digest_manifest();
    let mut extracted = paths
        .iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&archive.cargo_target)
                .context("archived executable escaped its producer target")?;
            let extracted = extracted_target.join(relative);
            anyhow::ensure!(
                bijux_dna_infra::hash_file_sha256(&extracted)? == original[path],
                "extracted executable differs from its source-bound producer: {}",
                extracted.display()
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let producer = path.metadata()?;
                let consumer = extracted.metadata()?;
                anyhow::ensure!(
                    producer.dev() != consumer.dev() || producer.ino() != consumer.ino(),
                    "extracted executable aliases its mutable compilation product: {}",
                    extracted.display()
                );
            }
            Ok(extracted)
        })
        .collect::<Result<BTreeSet<_>>>()?;
    extracted.extend([cargo_metadata, binary_metadata]);
    Ok(ArchivedExecutableSeal { extracted: ExecutableSeal::capture(&extracted)?, producer })
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

fn artifact_root_environment(root: &Path) -> Vec<(String, String)> {
    ["ARTIFACT_ROOT", "ISO_ROOT", "ARTIFACT_DIR"]
        .into_iter()
        .map(|key| (key.to_string(), root.display().to_string()))
        .collect()
}

fn full_archive_env(roots: &FullArchiveRoots, build_jobs: usize) -> Result<Vec<(String, String)>> {
    bijux_dna_infra::ensure_dir(&roots.cargo_target)?;
    let mut environment = artifact_root_environment(&roots.artifact_root);
    environment.extend([
        ("CARGO_TARGET_DIR".to_string(), roots.cargo_target.display().to_string()),
        ("CARGO_BUILD_JOBS".to_string(), build_jobs.to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ]);
    Ok(environment)
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
    let mut environment = vec![("NEXTEST_EXPERIMENTAL_LIBTEST_JSON".to_string(), "1".to_string())];
    environment.extend(artifact_root_environment(&outputs));
    environment.extend([
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
    ]);
    Ok(environment)
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
        "scratch_retained": true,
        "retention_scope": "exclusively_owned_compilation_cache",
        "compilation_cache_target": CompilationCache::target_under(&storage.scratch_root),
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
        || contract.shards.len() < 2
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

fn duration_milliseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
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
        runtime_dependency_report: "artifacts/ci/full-shards/runtime-dependencies.json".to_string(),
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

#[cfg(test)]
mod tests {
    use super::ci_output_path_under;
    use super::contract;
    use super::full_archive_args;
    use super::full_archive_roots;
    use super::full_shard_args;
    use super::full_shard_env;
    use super::full_shard_execution_scope;
    use super::full_shard_root_receipt;
    use super::full_shard_roots;
    use super::full_shard_storage_receipt;
    use super::isolated_build_tree;
    use super::materialize_full_shard_workspace;
    use super::observe_full_shard_runtime_dependencies;
    use super::observe_full_shard_runtime_dependencies_with_env;
    use super::read_ci_build_authority;
    use super::resolve_full_shard_storage_roots;
    use super::select_full_shards;
    use super::set_full_archive_readonly;
    use super::validate_ci_test_executor_observation;
    use super::validate_full_archive_roots;
    use super::validate_full_shard_contract;
    use super::validate_full_shard_root_set;
    use super::validate_full_shard_runtime_dependency_catalog;
    use super::FullArchiveRoots;
    use super::FullShard;
    use super::FullShardContract;
    use super::FullShardEphemera;
    use super::FullShardRoots;
    use super::FullShardRuntimeDependency;
    use super::FULL_SHARD_SCRATCH_ROOT_ENV;
    use crate::commands::ops::tooling::artifact_root_path;
    use crate::runtime::owned_artifacts::remove_owned_tree as remove_full_shard_ephemera;
    use crate::runtime::workspace::Workspace;
    use anyhow::Context;
    use std::collections::BTreeMap;

    #[test]
    fn full_shard_contract_covers_hash_space_exactly_once() {
        assert!(validate_full_shard_contract(&contract()).is_ok());
    }

    #[test]
    fn full_shard_contract_admits_hundreds_of_complete_hash_partitions() {
        for count in [100, 256] {
            let mut contract = contract();
            contract.shards = (1..=count)
                .map(|index| FullShard {
                    id: format!("hash-{index}-of-{count}"),
                    partition: format!("hash:{index}/{count}"),
                })
                .collect();
            assert!(validate_full_shard_contract(&contract).is_ok());
            contract.shards[0].partition = format!("hash:2/{count}");
            assert!(validate_full_shard_contract(&contract).is_err());
            contract.shards[0].partition = format!("hash:1/{}", count + 1);
            assert!(validate_full_shard_contract(&contract).is_err());
            contract.shards[0].partition = format!("hash:1/{count}");
            contract.archive_build_jobs = 17;
            assert!(validate_full_shard_contract(&contract).is_err());
        }
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
            "ARTIFACT_DIR",
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
        assert_eq!(first_env["ARTIFACT_DIR"], first_env["ARTIFACT_ROOT"]);
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
        let extraction = super::full_archive_extraction_args(&archive);
        assert!(extraction
            .windows(2)
            .any(|pair| pair[0] == "--list-type" && pair[1] == "binaries-only"));
        let native_inventory = crate::runtime::compiled_products::inventory_arguments(
            &super::archive_reuse_inputs(&archive.archive_extract)?,
            &storage.scratch_root.join("producer"),
        )?;
        assert!(native_inventory.iter().any(|arg| arg == "--cargo-metadata"));
        assert!(native_inventory
            .windows(2)
            .any(|pair| pair[0] == "--run-ignored" && pair[1] == "all"));
        assert!(!native_inventory.iter().any(|arg| [
            "--archive-file",
            "--extract-to",
            "--workspace",
            "--all-targets"
        ]
        .contains(&arg.as_str())));

        for shard in &contract.shards {
            let remap = storage.scratch_root.join("workspaces").join(&shard.id);
            let args = full_shard_args(
                shard,
                &super::archive_reuse_inputs(&archive.archive_extract)?,
                &remap,
                3,
            )?;
            assert!(args.windows(2).any(|pair| pair[0] == "--test-threads" && pair[1] == "3"));
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
    fn archive_executable_seal_rejects_changed_materialization_with_unchanged_archive(
    ) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir()?;
        let root = temp.path().canonicalize()?;
        let archive = FullArchiveRoots {
            artifact_root: root.join("archive"),
            archive_extract: root.join("extracted"),
            cargo_target: root.join("producer-target"),
            archive_file: root.join("archive.tar.zst"),
            build_log: root.join("build.log"),
        };
        let source = root.join("producer-source");
        let target = archive.archive_extract.join("target");
        let metadata = target.join("nextest");
        let binary = target.join("debug/deps/owner");
        std::fs::create_dir_all(&metadata)?;
        std::fs::create_dir_all(binary.parent().context("binary parent")?)?;
        std::fs::write(&binary, b"original executable")?;
        let producer_binary = archive.cargo_target.join("debug/deps/owner");
        std::fs::create_dir_all(producer_binary.parent().context("producer binary parent")?)?;
        std::fs::write(&producer_binary, b"original executable")?;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
        std::fs::write(
            metadata.join("cargo-metadata.json"),
            serde_json::to_vec(&serde_json::json!({
                "workspace_root": source, "target_directory": archive.cargo_target,
            }))?,
        )?;
        std::fs::write(
            metadata.join("binaries-metadata.json"),
            serde_json::to_vec(&serde_json::json!({
                "rust-build-meta": {"target-directory": archive.cargo_target, "non-test-binaries": {}},
                "rust-binaries": {"owner": {"binary-id": "owner", "binary-path": archive.cargo_target.join("debug/deps/owner")}},
            }))?,
        )?;
        std::fs::write(&archive.archive_file, b"unchanged source-bound archive")?;
        let archive_sha256 = bijux_dna_infra::hash_file_sha256(&archive.archive_file)?;
        super::seal_full_archive_extraction(&archive.archive_extract)?;
        let seal = super::capture_archive_executable_seal(&archive, &source)?;
        seal.extracted.require_readonly()?;
        assert!(seal.extracted.verify().unchanged);
        assert!(seal.producer.verify().unchanged);
        let original_permissions = std::fs::metadata(&binary)?.permissions();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
        std::fs::write(&binary, b"modified executable")?;
        std::fs::set_permissions(&binary, original_permissions)?;
        assert_eq!(bijux_dna_infra::hash_file_sha256(&archive.archive_file)?, archive_sha256);
        let observed = seal.extracted.verify();
        assert!(!observed.unchanged);
        assert_eq!(observed.receipt["changed_inputs"][0], binary.display().to_string());
        assert!(seal.producer.verify().unchanged);
        std::fs::write(&producer_binary, b"changed retained producer")?;
        assert!(!seal.producer.verify().unchanged);
        let cargo_metadata = std::fs::read(metadata.join("cargo-metadata.json"))?;
        let binary_metadata = std::fs::read(metadata.join("binaries-metadata.json"))?;
        super::remove_full_shard_ephemera(&archive.archive_extract)?;
        std::fs::create_dir_all(&metadata)?;
        std::fs::create_dir_all(binary.parent().context("binary parent")?)?;
        std::fs::write(metadata.join("cargo-metadata.json"), cargo_metadata)?;
        std::fs::write(metadata.join("binaries-metadata.json"), binary_metadata)?;
        std::fs::hard_link(&producer_binary, &binary)?;
        super::seal_full_archive_extraction(&archive.archive_extract)?;
        assert!(super::capture_archive_executable_seal(&archive, &source).is_err());
        super::remove_full_shard_ephemera(&archive.archive_extract)?;
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
