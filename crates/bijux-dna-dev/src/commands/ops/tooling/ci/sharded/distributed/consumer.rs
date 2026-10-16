//! Execute one declared partition solely from a transported, admitted archive.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use crate::application::ci::{
    CompiledWorkspaceExecutables, CoverageAttestation, ExpectedTestInventory, ShardDiscoveryPlan,
};
use crate::commands::ops::tooling::ci::full_execution::{self, ExecutionBudget, ShardInputs};
use crate::runtime::crate_gate_inputs::{CrateGateProducerIdentity, CrateGateProofInputs};
use crate::runtime::workspace::Workspace;

use super::super::archive::{
    archive_reuse_inputs, full_archive_env, full_archive_extraction_args,
    seal_full_archive_extraction,
};
use super::super::contract::FullShard;
use super::super::storage::{full_shard_roots, FullArchiveRoots, FullShardStorageRoots};
use super::protocol::{imported_seal, remap_inventory};
use super::{command, load_producer, Session};

pub(super) fn consume(
    workspace: &Workspace,
    root: &Path,
    reports: &Path,
    partition: &str,
    session: &mut Session,
    receipt: &mut Value,
) -> Result<()> {
    let producer = load_producer(root, session, true)?;
    let selected = producer
        .matrix
        .include
        .iter()
        .find(|row| row.partition == partition)
        .context("partition is absent from producer contract")?;
    receipt["producer_sha256"] =
        json!(bijux_dna_infra::hash_file_sha256(&root.join("build/producer.json"))?);
    receipt["partition"] = json!(partition);
    receipt["id"] = json!(selected.id);
    let run = session.scratch.join("run");
    ensure!(!run.try_exists()?, "consumer scratch is not fresh");
    std::fs::create_dir_all(&run)?;
    let roots = FullArchiveRoots {
        artifact_root: run.join("runtime"),
        archive_extract: run.join("extracted"),
        cargo_target: run.join("unused-cargo-target"),
        archive_file: root.join("build/archive.tar.zst"),
        build_log: reports.join("archive-extract.log"),
    };
    let envs = full_archive_env(&roots, 1)?;
    super::super::archive::prepare_full_archive_extraction(&roots.archive_extract)?;
    command(workspace, reports, "archive-extract", full_archive_extraction_args(&roots), envs)?;
    seal_full_archive_extraction(&roots.archive_extract)?;
    let target = roots.archive_extract.join("target");
    let seal = imported_seal(&target, &producer.inputs)?;
    let original_inventory = std::fs::read(root.join("build/inventory.json"))?;
    let inventory_bytes =
        remap_inventory(&original_inventory, &producer, &workspace.root, &target)?;
    let inventory_path = reports.join("inventory-remapped.json");
    std::fs::write(&inventory_path, &inventory_bytes)?;
    let inventory = ExpectedTestInventory::parse(&inventory_bytes)?;
    let executables = CompiledWorkspaceExecutables::parse(&inventory_bytes)?;
    let plan = ShardDiscoveryPlan::from_inventory(&inventory, partition, "0.9.100")?;
    receipt["expected_case_count"] = json!(plan.expected_case_count);
    let admission = session.admission.as_ref().context("consumer source admission absent")?;
    let proof = CrateGateProofInputs::capture(
        &inventory_path,
        &workspace.root,
        &target,
        CrateGateProducerIdentity::new(
            &producer.source_commit,
            &admission.snapshot.patch_sha256(),
        )?,
    )?;
    let shard = FullShard { id: selected.id.clone(), partition: partition.to_owned() };
    let storage = FullShardStorageRoots {
        retained_report_root: reports.to_path_buf(),
        scratch_root: run.clone(),
    };
    let shard_roots = full_shard_roots(&storage, &admission.contract, &shard)?;
    let budget = ExecutionBudget::resolve(1)?;
    let reports_native = reports.join("native");
    std::fs::create_dir_all(&reports_native)?;
    let reuse = archive_reuse_inputs(&roots.archive_extract)?;
    let execution = full_execution::run_shards(
        ShardInputs {
            workspace,
            source_condition: &admission.condition,
            shards: std::slice::from_ref(&shard),
            roots: std::slice::from_ref(&shard_roots),
            plans: std::slice::from_ref(&plan),
            inventory: &inventory,
            inventory_bytes: &inventory_bytes,
            snapshot: &admission.snapshot,
            scratch_root: &run,
            report_root: &reports_native,
            reuse: &reuse,
            executables: &executables,
            compiled_target: &target,
            crate_gate_inputs: &proof,
        },
        &budget,
    );
    let observed = seal.verify();
    receipt["extracted_input_postcondition"] = observed.receipt;
    ensure!(observed.unchanged, "imported executable inputs changed during partition execution");
    let execution = execution?;
    ensure!(execution.outputs.len() == 1, "consumer returned a foreign terminal roster");
    let terminal = &execution.outputs[0];
    let coverage = CoverageAttestation::reconcile_terminals(
        &inventory,
        [(shard.id.as_str(), terminal.events(), &plan)],
    );
    receipt["coverage"] = serde_json::to_value(&coverage)?;
    receipt["resources"] = serde_json::to_value(budget)?;
    receipt["native_execution_observed"] = json!(!terminal.is_empty());
    receipt["native_exit_code"] = json!(terminal.exit_code());
    if let Some(events) = terminal.events() {
        std::fs::write(reports.join("events.jsonl"), events)?;
        receipt["events_sha256"] = json!(bijux_dna_infra::hash_bytes_sha256(events));
    }
    receipt["all_passed"] = json!(terminal.succeeded() && coverage.selected_shards_valid);
    ensure!(
        terminal.succeeded() && coverage.selected_shards_valid,
        "partition has failed or invalid native evidence"
    );
    Ok(())
}
