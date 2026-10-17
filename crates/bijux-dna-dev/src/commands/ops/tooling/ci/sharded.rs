//! Full-shard source/product transaction and terminal evidence.
pub(super) mod archive;
pub(super) mod contract;
mod distributed;
mod preflight;
pub(super) mod storage;

use crate::application::ci::CompiledWorkspaceExecutables;
use crate::application::ci::CoverageAttestation;
use crate::application::ci::ExpectedTestInventory;
use crate::application::ci::ShardDiscoveryPlan;
use crate::commands::ops::tooling::ci::ci_output_path;
use crate::commands::ops::tooling::ci::full_execution;
use crate::commands::ops::tooling::ci::read_ci_build_authority;
use crate::commands::ops::tooling::ci::sharded::archive::archive_reuse_inputs;
use crate::commands::ops::tooling::ci::sharded::archive::capture_archive_executable_seal;
use crate::commands::ops::tooling::ci::sharded::archive::full_archive_args;
use crate::commands::ops::tooling::ci::sharded::archive::full_archive_env;
use crate::commands::ops::tooling::ci::sharded::archive::full_archive_extraction_args;
use crate::commands::ops::tooling::ci::sharded::archive::prepare_full_archive_extraction;
use crate::commands::ops::tooling::ci::sharded::archive::prepare_full_archive_roots;
use crate::commands::ops::tooling::ci::sharded::archive::seal_full_archive_extraction;
use crate::commands::ops::tooling::ci::sharded::archive::set_full_archive_readonly;
use crate::commands::ops::tooling::ci::sharded::contract::full_shard_execution_scope;
use crate::commands::ops::tooling::ci::sharded::contract::select_full_shards;
use crate::commands::ops::tooling::ci::sharded::contract::validate_full_shard_contract;
use crate::commands::ops::tooling::ci::sharded::contract::FullShardContract;
use crate::commands::ops::tooling::ci::sharded::preflight::observe_full_shard_runtime_dependencies;
use crate::commands::ops::tooling::ci::sharded::preflight::require_ci_test_executor;
use crate::commands::ops::tooling::ci::sharded::preflight::validate_full_shard_runtime_dependency_catalog;
use crate::commands::ops::tooling::ci::sharded::storage::full_archive_roots;
use crate::commands::ops::tooling::ci::sharded::storage::full_shard_root_receipt;
use crate::commands::ops::tooling::ci::sharded::storage::full_shard_roots;
use crate::commands::ops::tooling::ci::sharded::storage::full_shard_storage_receipt;
use crate::commands::ops::tooling::ci::sharded::storage::materialize_full_shard_workspace;
use crate::commands::ops::tooling::ci::sharded::storage::observe_full_shard_cleanup;
use crate::commands::ops::tooling::ci::sharded::storage::paths_overlap;
use crate::commands::ops::tooling::ci::sharded::storage::resolve_full_shard_storage_roots;
use crate::commands::ops::tooling::ci::sharded::storage::validate_full_archive_roots;
use crate::commands::ops::tooling::ci::sharded::storage::validate_full_shard_root_set;
use crate::commands::ops::tooling::ci::sharded::storage::FullShardEphemera;
use crate::commands::ops::tooling::ci::sharded::storage::FULL_SHARD_SCRATCH_ROOT_ENV;
use crate::commands::ops::tooling::ci::source_worktree_clean;
use crate::commands::ops::tooling::ensure_help_only;
use crate::commands::ops::tooling::write_json_pretty;
use crate::model::ops::OpsCommandOutcome;
use crate::runtime::compilation_cache::CompilationCache;
use crate::runtime::compiled_products::discover_native_inventory;
use crate::runtime::compiled_products::LocalCompiledProducts;
use crate::runtime::process::ParallelProcessSpec;
use crate::runtime::process::ProcessRunner;
use crate::runtime::source_snapshot::IsolatedSourceIdentity;
use crate::runtime::source_snapshot::TrackedSourceSnapshot;
use crate::runtime::workspace::Workspace;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

pub(in super::super::super) fn tooling_ci_test_all_sharded(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args.first().is_some_and(|arg| arg.starts_with("--github-")) {
        return distributed::run(workspace, args);
    }
    if matches!(args, [single] if single == "--help" || single == "-h") {
        ensure_help_only("ci-test-all-sharded", args)?;
    }
    let started = Instant::now();
    let (local_products, args) = match args.split_first() {
        Some((flag, remaining)) if flag == "--local-products" => (true, remaining),
        _ => (false, args),
    };
    let contract_path = workspace.path("configs/ci/full-shards.json");
    let preflight_only = matches!(args, [single] if single == "--preflight-only");
    let receipt_path = ci_output_path(workspace, "artifacts/ci/full-gate-timing.json")?;
    // Runtime admission must replace stale successful evidence before any early refusal.
    // A successful preflight alone never certifies test execution.
    let (
        contract,
        selected_shards,
        storage,
        build_authority,
        build_authority_sha256,
        runtime_dependencies,
    ) = crate::runtime::gate_receipt::run_reported(
        &receipt_path,
        serde_json::json!({
            "schema_version": "bijux.ci.full_gate_timing.v4",
            "operation": "runtime_admission",
            "test_execution_started": false,
            "all_passed": false,
        }),
        started,
        &mut (),
        |(), receipt| {
            let contract: FullShardContract = serde_json::from_slice(&fs::read(&contract_path)?)
                .with_context(|| format!("parse {}", contract_path.display()))?;
            validate_full_shard_contract(&contract)?;
            let selected_shards =
                select_full_shards(&contract, if preflight_only { &[] } else { args })?;
            validate_full_shard_runtime_dependency_catalog(
                workspace,
                &contract.runtime_dependencies,
            )?;

            let storage = resolve_full_shard_storage_roots(
                workspace,
                &contract,
                std::env::var_os(FULL_SHARD_SCRATCH_ROOT_ENV).map(PathBuf::from),
            )?;
            let build_authority_path = workspace.path(&contract.build_authority);
            let build_authority = read_ci_build_authority(&build_authority_path)?;
            require_ci_test_executor(workspace, &build_authority)?;
            let build_authority_sha256 = bijux_dna_infra::hash_file_sha256(&build_authority_path)?;
            bijux_dna_infra::ensure_dir(&storage.retained_report_root)?;
            let runtime_dependencies = observe_full_shard_runtime_dependencies(
                workspace,
                &contract.runtime_dependencies,
                &ci_output_path(workspace, &contract.runtime_dependency_report)?,
            )?;
            receipt["runtime_dependencies"] = serde_json::json!(&runtime_dependencies);
            receipt["runtime_admission_passed"] = serde_json::json!(true);
            Ok((
                contract,
                selected_shards,
                storage,
                build_authority,
                build_authority_sha256,
                runtime_dependencies,
            ))
        },
        |(), _| Ok(()),
    )?;
    let report_root = storage.retained_report_root.clone();
    let build_authority_path = workspace.path(&contract.build_authority);
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
                    &expected_source,
                    &full_archive_env(&archive_roots, archive_build_jobs)?,
                    &report_root,
                    execution_budget.worker_budget,
                )?
            } else {
                discover_native_inventory(
                    workspace,
                    &archive_workspace,
                    &expected_source,
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
                    workspace: &compiled_source,
                    source_condition: &expected_source,
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
