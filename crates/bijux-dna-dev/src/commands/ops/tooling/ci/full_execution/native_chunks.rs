//! Bounded native startup and terminal sequencing within each exact declared partition.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Output;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use serde_json::json;

use crate::application::ci::{BinaryDiscoveryPlan, CoverageAttestation, ShardDiscoveryPlan};
use crate::runtime::compiled_products::{
    native_child_records, publish_native_metadata, MetadataInput,
};
use crate::runtime::process::{
    DependentProcessSpec, ParallelProcessOutput, ParallelProcessSpec, ProcessRunner,
    WeightedProcessSpec,
};

use crate::runtime::source_snapshot::IsolatedSourceIdentity;
use crate::runtime::workspace::Workspace;

use super::super::sharded::archive::full_shard_args;
use super::super::sharded::storage::{full_shard_env, materialize_full_shard_workspace};
use super::{ExecutionBudget, ShardExecution, ShardInputs, ShardTerminal};

struct NativeChunk {
    shard_index: usize,
    label: String,
    metadata: Vec<u8>,
    binaries: BTreeSet<String>,
    resource_units: usize,
    terminal_dependencies: Vec<String>,
}

fn chunks(
    metadata: &[u8],
    shard_ids: &[String],
    plans: &[ShardDiscoveryPlan],
    budget: &ExecutionBudget,
) -> Result<Vec<NativeChunk>> {
    ensure!(
        shard_ids.len() == plans.len()
            && plans.len() == budget.selected_shards
            && budget.worker_budget >= 2
            && budget.process_jobs > 0
            && budget.test_threads_per_process > 0
            && budget.test_threads_per_process <= budget.worker_budget
            && shard_ids.iter().collect::<BTreeSet<_>>().len() == shard_ids.len(),
        "native partition roster/capacity is invalid"
    );
    let per_command_budget = (budget.worker_budget / budget.process_jobs)
        .max(2)
        .max(budget.test_threads_per_process)
        .min(budget.worker_budget);
    // Validate producer metadata even when every selected assignment is empty.
    BinaryDiscoveryPlan::new(metadata, per_command_budget)?;
    let groups = plans
        .iter()
        .map(|plan| {
            if plan.expected_case_count == 0 {
                ensure!(
                    plan.selected_binary_ids().is_empty(),
                    "empty plan selects native binaries"
                );
                Ok(Vec::new())
            } else {
                Ok(BinaryDiscoveryPlan::selected(
                    metadata,
                    Some(&plan.selected_binary_ids()),
                    per_command_budget,
                )?
                .groups)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let mut chunks = Vec::new();
    let longest = groups.iter().map(Vec::len).max().unwrap_or_default();
    let mut predecessors = BTreeMap::new();
    // Interleave ready partitions; terminal edges prevent same-workspace groups from overlapping.
    for index in 0..longest {
        for (shard_index, plan) in groups.iter().enumerate() {
            if let Some(group) = plan.get(index) {
                let label = format!("{}-{}", shard_ids[shard_index], group.label);
                let terminal_dependencies =
                    predecessors.insert(shard_index, label.clone()).into_iter().collect();
                chunks.push(NativeChunk {
                    shard_index,
                    label,
                    metadata: group.metadata.clone(),
                    binaries: group.binaries.clone(),
                    resource_units: group
                        .worker_demand(budget.test_threads_per_process, budget.worker_budget)?,
                    terminal_dependencies,
                });
            }
        }
    }
    Ok(chunks)
}

fn merge_outputs(
    shard_ids: &[String],
    plans: &[ShardDiscoveryPlan],
    chunks: &[NativeChunk],
    outputs: Vec<ParallelProcessOutput>,
) -> Result<Vec<ShardTerminal>> {
    ensure!(
        !shard_ids.is_empty()
            && shard_ids.len() == plans.len()
            && shard_ids.iter().collect::<BTreeSet<_>>().len() == shard_ids.len(),
        "partition terminal roster differs from plans"
    );
    ensure!(
        chunks.iter().all(|chunk| chunk.shard_index < plans.len()),
        "foreign native chunk partition"
    );
    let mut observed = BTreeMap::new();
    for output in outputs {
        ensure!(
            chunks.iter().any(|chunk| chunk.label == output.label),
            "unrecognized native execution chunk"
        );
        ensure!(
            observed.insert(output.label.clone(), output).is_none(),
            "duplicate native execution chunk"
        );
    }
    ensure!(observed.len() == chunks.len(), "missing terminal native execution chunk");
    shard_ids
        .iter()
        .enumerate()
        .map(|(shard_index, label)| {
            let selected =
                chunks.iter().filter(|chunk| chunk.shard_index == shard_index).collect::<Vec<_>>();
            if plans[shard_index].expected_case_count == 0 {
                ensure!(selected.is_empty(), "expected-empty partition has native chunks");
                return Ok(ShardTerminal::ExpectedEmpty { label: label.clone() });
            }
            ensure!(!selected.is_empty(), "partition has no terminal native chunks");
            let mut status = None;
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut duration = Duration::ZERO;
            for chunk in selected {
                let output = observed.remove(&chunk.label).context("native chunk output absent")?;
                if status.is_none() || !output.output.status.success() {
                    status = Some(output.output.status);
                }
                stdout.extend_from_slice(&output.output.stdout);
                if !stdout.is_empty() && stdout.last() != Some(&b'\n') {
                    stdout.push(b'\n');
                }
                stderr.extend_from_slice(&output.output.stderr);
                if !stderr.is_empty() && stderr.last() != Some(&b'\n') {
                    stderr.push(b'\n');
                }
                duration = duration
                    .checked_add(output.duration)
                    .context("partition chunk duration overflow")?;
            }
            Ok(ShardTerminal::Executed(ParallelProcessOutput {
                label: label.clone(),
                duration,
                output: Output {
                    status: status.context("native chunk verdict absent")?,
                    stdout,
                    stderr,
                },
            }))
        })
        .collect()
}

/// Shared chunk admission and weighted execution; preparation retains its caller-owned authority.
fn run_native_chunks<F>(
    workspace: &Workspace,
    plan: &[NativeChunk],
    budget: &ExecutionBudget,
    report_root: &std::path::Path,
    prepare: F,
) -> Result<Vec<ParallelProcessOutput>>
where
    F: Fn(&ParallelProcessSpec) -> Result<ParallelProcessSpec> + Sync,
{
    let specifications = plan
        .iter()
        .map(|chunk| WeightedProcessSpec {
            process: DependentProcessSpec {
                process: ParallelProcessSpec {
                    label: chunk.label.clone(),
                    program: String::new(),
                    args: vec![],
                    envs: vec![],
                    current_dir: None,
                },
                dependencies: vec![],
            },
            resource_units: chunk.resource_units,
            terminal_dependencies: chunk.terminal_dependencies.clone(),
        })
        .collect();
    ProcessRunner::new(workspace).run_prepared_weighted_parallel_logged(
        specifications,
        budget.process_jobs,
        budget.worker_budget,
        report_root,
        prepare,
    )
}

pub(super) fn run_shards(
    inputs: ShardInputs<'_>,
    budget: &ExecutionBudget,
) -> Result<ShardExecution> {
    ensure!(
        inputs.shards.len() == inputs.roots.len() && inputs.shards.len() == inputs.plans.len(),
        "full-shard execution inputs differ from admitted roster"
    );
    let captured_inventory =
        crate::application::ci::ExpectedTestInventory::parse(inputs.inventory_bytes)?;
    for (shard, plan) in inputs.shards.iter().zip(inputs.plans) {
        plan.require_partition(&captured_inventory, &shard.partition)?;
        plan.verify_inventory(inputs.inventory)?;
    }
    let expected_source = IsolatedSourceIdentity::capture(inputs.workspace)?;
    expected_source.require_snapshot(inputs.snapshot)?;
    let shard_ids = inputs.shards.iter().map(|shard| shard.id.clone()).collect::<Vec<_>>();
    let original_metadata = std::fs::read(inputs.reuse.binaries_metadata())?;
    let producer_sha256 = bijux_dna_infra::hash_bytes_sha256(&original_metadata);
    let plan = chunks(&original_metadata, &shard_ids, inputs.plans, budget)?;
    let workspaces = shard_ids
        .iter()
        .map(|id| inputs.scratch_root.join("workspaces").join(id))
        .collect::<Vec<_>>();
    let mut metadata_inputs = BTreeMap::<String, MetadataInput>::new();
    for chunk in &plan {
        metadata_inputs.insert(
            chunk.label.clone(),
            publish_native_metadata(
                &inputs.report_root.join(format!("{}.binaries.json", chunk.label)),
                &chunk.metadata,
            )?,
        );
    }
    let receipt_path = inputs.report_root.join("native-execution-chunks.receipt.json");
    let mut receipt = json!({"schema_version":"bijux.ci.bounded_native_execution.v1","all_passed":false,
        "source_condition":expected_source.descriptor(),
        "source_commit":inputs.snapshot.commit(),"source_patch_sha256":inputs.snapshot.patch_sha256(),
        "producer_inventory_sha256":bijux_dna_infra::hash_bytes_sha256(inputs.inventory_bytes),
        "partition_plans":inputs.plans,
        "expected_empty_partitions":shard_ids.iter().zip(inputs.plans).filter(|(_,plan)|plan.expected_case_count==0).map(|(id,_)|id).collect::<Vec<_>>(),
        "producer_binary_metadata_sha256":producer_sha256,"native_worker_budget":budget.worker_budget,
        "worker_scope":"ordinary/ignored native listing and declared case workers; parent runtimes and nested tools have no hard physical CPU/process cap",
        "metadata_inputs":metadata_inputs,
        "expected_chunks":plan.iter().map(|chunk|json!({"label":chunk.label,"shard":shard_ids[chunk.shard_index],"binaries":chunk.binaries,"resource_units":chunk.resource_units,"terminal_dependencies":chunk.terminal_dependencies})).collect::<Vec<_>>()});
    bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
    let materialized_sources = Mutex::new(BTreeSet::new());
    let preparation = Mutex::new(BTreeMap::new());
    let proof_handles = Mutex::new(BTreeMap::new());
    let outputs =
        run_native_chunks(inputs.workspace, &plan, budget, inputs.report_root, |admitted| {
            let chunk = plan
                .iter()
                .find(|chunk| chunk.label == admitted.label)
                .context("admitted native chunk absent")?;
            let index = chunk.shard_index;
            let started = Instant::now();
            if chunk.terminal_dependencies.is_empty() {
                materialized_sources
                    .lock()
                    .map_err(|_| anyhow::anyhow!("native source receipt lock poisoned"))?
                    .insert(index);
                let workspace = materialize_full_shard_workspace(
                    inputs.snapshot,
                    inputs.scratch_root,
                    &shard_ids[index],
                )?;
                ensure!(
                    workspace == workspaces[index],
                    "native chunk preparation changed workspace authority"
                );
                expected_source.verify(&Workspace { root: workspace.clone() })?;
                let published = inputs.crate_gate_inputs.publish_for_consumer(&workspace)?;
                proof_handles
                    .lock()
                    .map_err(|_| anyhow::anyhow!("native proof receipt lock poisoned"))?
                    .insert(index, published);
                preparation
                    .lock()
                    .map_err(|_| anyhow::anyhow!("native preparation receipt lock poisoned"))?
                    .insert(
                        shard_ids[index].clone(),
                        super::super::duration_milliseconds(started.elapsed()),
                    );
            }
            let subset =
                inputs.reuse.with_binaries_metadata(metadata_inputs[&chunk.label].path.clone())?;
            metadata_inputs[&chunk.label].verify_unchanged()?;
            let mut args = full_shard_args(
                &inputs.shards[index],
                &subset,
                &workspaces[index],
                budget.test_threads_per_process,
            )?;
            args.extend(["--filterset".into(), inputs.plans[index].filterset.clone()]);
            let mut envs = full_shard_env(&inputs.roots[index], &workspaces[index])?;
            envs.extend(inputs.executables.remapped_environment(inputs.compiled_target)?);
            envs.push(
                proof_handles
                    .lock()
                    .map_err(|_| anyhow::anyhow!("native proof receipt lock poisoned"))?
                    .get(&index)
                    .context("native shard proof absent")?
                    .environment()?,
            );
            Ok(ParallelProcessSpec {
                label: chunk.label.clone(),
                program: "cargo".into(),
                args,
                envs,
                current_dir: Some(workspaces[index].clone()),
            })
        });
    let handles = proof_handles
        .into_inner()
        .map_err(|_| anyhow::anyhow!("native proof receipt lock poisoned"))?;
    let mut crate_gate_proof_inputs = BTreeMap::new();
    let mut failures = Vec::new();
    let materialized = materialized_sources
        .into_inner()
        .map_err(|_| anyhow::anyhow!("native source receipt lock poisoned"))?;
    let source_postconditions = materialized
        .into_iter()
        .map(|index| {
            let observed =
                expected_source.postcondition(&Workspace { root: workspaces[index].clone() });
            if observed["unchanged"] != true {
                failures.push(format!("{} source: {}", shard_ids[index], observed["error"]));
            }
            observed
        })
        .collect::<Vec<_>>();
    receipt["source_postconditions"] = json!(source_postconditions);
    if let Err(error) = expected_source.verify(inputs.workspace) {
        failures.push(format!("coordinator source: {error:#}"));
    }
    for (index, published) in handles {
        if let Err(error) = published.verify_unchanged() {
            failures.push(format!("{}: {error:#}", shard_ids[index]));
        }
        crate_gate_proof_inputs.insert(shard_ids[index].clone(), published.provenance);
    }
    if let Err(error) = inputs.crate_gate_inputs.verify_unchanged() {
        failures.push(format!("producer: {error:#}"));
    }
    for input in metadata_inputs.values() {
        if let Err(error) = input.verify_unchanged() {
            failures.push(format!("{}: {error:#}", input.path.display()));
        }
    }
    if bijux_dna_infra::hash_file_sha256(inputs.reuse.binaries_metadata())? != producer_sha256 {
        failures.push("producer binary metadata changed".into());
    }
    receipt["terminal_chunks"] = json!(native_child_records(
        plan.iter().map(|chunk| chunk.label.as_str()),
        inputs.report_root
    )?);
    receipt["physical_input_errors"] = json!(failures);
    receipt["execution_error"] = json!(outputs.as_ref().err().map(|error| format!("{error:#}")));
    if let Ok(outputs) = &outputs {
        receipt["chunks"] = json!(outputs.iter().map(|output| json!({"label":output.label,"exit_code":output.output.status.code(),"duration_milliseconds":output.duration.as_millis()})).collect::<Vec<_>>());
    }
    let merged =
        outputs.and_then(|outputs| merge_outputs(&shard_ids, inputs.plans, &plan, outputs));
    if let Ok(outputs) = &merged {
        receipt["partition_terminals"] = json!(outputs
            .iter()
            .map(|output| json!({
                "id":output.label(),
                "state":if output.is_empty(){"expected_empty"}else{"executed"},
                "exit_code":output.exit_code(),
                "native_execution_observed":!output.is_empty()
            }))
            .collect::<Vec<_>>());
        let coverage = CoverageAttestation::reconcile_terminals(
            inputs.inventory,
            outputs
                .iter()
                .zip(inputs.plans)
                .map(|(output, plan)| (output.label(), output.events(), plan)),
        );
        receipt["all_passed"] = json!(
            failures.is_empty()
                && coverage.selected_shards_valid
                && outputs.iter().all(ShardTerminal::succeeded)
        );
        receipt["coverage"] = json!(coverage);
    }
    receipt["aggregation_error"] = json!(merged.as_ref().err().map(|error| format!("{error:#}")));
    bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
    ensure!(failures.is_empty(), "native chunk physical inputs changed: {}", failures.join("; "));
    let merged = merged?;
    for output in &merged {
        bijux_dna_infra::write_bytes(
            inputs.report_root.join(format!("{}.stdout.log", output.label())),
            output.stdout(),
        )?;
        bijux_dna_infra::write_bytes(
            inputs.report_root.join(format!("{}.stderr.log", output.label())),
            output.stderr(),
        )?;
    }
    let preparation_milliseconds = preparation
        .into_inner()
        .map_err(|_| anyhow::anyhow!("native preparation receipt lock poisoned"))?;
    Ok(ShardExecution {
        workspaces,
        outputs: merged,
        preparation_milliseconds,
        crate_gate_proof_inputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ci::ExpectedTestInventory;
    use twox_hash::XxHash64;

    struct PartitionFixture {
        metadata: Vec<u8>,
        inventory: ExpectedTestInventory,
        shard_ids: Vec<String>,
        shard_plans: Vec<ShardDiscoveryPlan>,
        budget: ExecutionBudget,
    }

    fn fixture() -> Result<PartitionFixture> {
        let mut binaries = serde_json::Map::new();
        let mut suites = serde_json::Map::new();
        for index in 0..8 {
            let id = format!("owner::binary{index}");
            let name = format!("binary{index}");
            let binary = json!({"binary-id":id,"binary-name":name,"package-id":"owner-package","kind":"test","binary-path":format!("/compiled/{name}"),"build-platform":"target"});
            let mut suite = binary.clone();
            suite["package-name"] = json!("owner");
            suite["status"] = json!("listed");
            suite["testcases"] = json!((0..32)
                .map(|case| (
                    format!("case-{case}"),
                    json!({"ignored":case%7==0,"filter-match":{"status":"matches"}})
                ))
                .collect::<BTreeMap<_, _>>());
            binaries.insert(id.clone(), binary);
            suites.insert(id, suite);
        }
        let metadata = serde_json::to_vec(
            &json!({"rust-build-meta":{"target-directory":"/compiled"},"rust-binaries":binaries}),
        )?;
        let inventory = ExpectedTestInventory::parse(&serde_json::to_vec(
            &json!({"test-count":256,"rust-suites":suites}),
        )?)?;
        let ids = (1..=5).map(|index| format!("hash-{index}-of-5")).collect::<Vec<_>>();
        let plans = (1..=5)
            .map(|index| {
                ShardDiscoveryPlan::from_inventory(
                    &inventory,
                    &format!("hash:{index}/5"),
                    "0.9.100",
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let budget = ExecutionBudget {
            detected_cpu_capacity: 10,
            worker_budget: 8,
            allocation_origin: "explicit_worker_budget",
            selected_shards: 5,
            process_jobs: 4,
            test_threads_per_process: 2,
            maximum_case_workers: 8,
            preparation_jobs: 4,
        };
        Ok(PartitionFixture { metadata, inventory, shard_ids: ids, shard_plans: plans, budget })
    }

    fn completed(chunk: &NativeChunk, partitions: u64) -> Result<ParallelProcessOutput> {
        let metadata: serde_json::Value = serde_json::from_slice(&chunk.metadata)?;
        let mut records = Vec::new();
        for binary in metadata["rust-binaries"].as_object().context("fixture binaries")?.values() {
            let name = binary["binary-name"].as_str().context("fixture name")?;
            let cases = (0..32)
                .map(|case| format!("case-{case}"))
                .filter(|name| {
                    XxHash64::oneshot(0, name.as_bytes()) % partitions == chunk.shard_index as u64
                })
                .collect::<Vec<_>>();
            let suite = json!({"crate":"owner","test_binary":name,"kind":"test"});
            records.push(
                json!({"type":"suite","event":"started","test_count":cases.len(),"nextest":suite}),
            );
            for case in &cases {
                let wire = format!("owner::{name}${case}");
                records.push(json!({"type":"test","event":"started","name":wire}));
                records.push(json!({"type":"test","event":"ok","name":wire}));
            }
            records.push(json!({"type":"suite","event":"ok","passed":cases.len(),"failed":0,"ignored":0,"measured":0,"nextest":suite}));
        }
        let mut stdout = Vec::new();
        for record in records {
            stdout.extend(serde_json::to_vec(&record)?);
            stdout.push(b'\n');
        }
        let status = std::process::Command::new("sh").args(["-c", "exit 0"]).status()?;
        Ok(ParallelProcessOutput {
            label: chunk.label.clone(),
            duration: Duration::ZERO,
            output: Output { status, stdout, stderr: vec![] },
        })
    }

    #[test]
    fn bounded_chunk_streams_preserve_all_five_exact_partitions_and_ignored_cases() -> Result<()> {
        let PartitionFixture { metadata, inventory, shard_ids: ids, shard_plans: plans, budget } =
            fixture()?;
        let planned = chunks(&metadata, &ids, &plans, &budget)?;
        assert_eq!(planned.len(), 40);
        for (index, shard_plan) in plans.iter().enumerate() {
            let selected =
                planned.iter().filter(|chunk| chunk.shard_index == index).collect::<Vec<_>>();
            let mut union = BTreeSet::new();
            for (position, chunk) in selected.iter().enumerate() {
                assert_eq!(chunk.resource_units, 2);
                assert_eq!(chunk.binaries.len(), 1);
                for binary in &chunk.binaries {
                    assert!(union.insert(binary.clone()), "binary repeated within partition");
                }
                if position == 0 {
                    assert!(chunk.terminal_dependencies.is_empty());
                } else {
                    assert_eq!(chunk.terminal_dependencies, [selected[position - 1].label.clone()]);
                }
            }
            assert_eq!(union, shard_plan.selected_binary_ids());
        }
        let mut observed =
            planned.iter().map(|chunk| completed(chunk, 5)).collect::<Result<Vec<_>>>()?;
        observed.reverse();
        let merged = merge_outputs(&ids, &plans, &planned, observed)?;
        let coverage = CoverageAttestation::reconcile_terminals(
            &inventory,
            merged.iter().zip(&plans).map(|(output, plan)| (output.label(), output.events(), plan)),
        );
        assert!(coverage.complete && coverage.selected_shards_valid);
        assert_eq!(merged.len(), 5);
        assert!(inventory.ignored_case_count() > 0);

        // A complete partition roster can contain no cases in some assignments.
        // This maintained metadata fixture has only 32 distinct names, even though
        // those names occur in eight binaries, so 100 assignments must include empties.
        let partition_count = 100;
        let expanded_ids = (1..=partition_count)
            .map(|index| format!("hash-{index}-of-{partition_count}"))
            .collect::<Vec<_>>();
        let expanded_plans = (1..=partition_count)
            .map(|index| {
                ShardDiscoveryPlan::from_inventory(
                    &inventory,
                    &format!("hash:{index}/{partition_count}"),
                    "0.9.100",
                )
            })
            .collect::<Result<Vec<_>>>()?;
        assert!(expanded_plans.iter().any(|plan| plan.expected_case_count == 0));
        assert!(expanded_plans.iter().any(|plan| plan.expected_case_count > 0));
        assert_eq!(
            expanded_plans.iter().map(|plan| plan.expected_case_count).sum::<usize>(),
            inventory.case_count()
        );
        std::io::Write::write_all(
            &mut std::io::stdout(),
            b"EMPTY_PARTITION_BASELINE_CONTROL_PASS\n",
        )?;
        let mut expanded_budget = budget;
        expanded_budget.selected_shards = partition_count;
        let expanded = chunks(&metadata, &expanded_ids, &expanded_plans, &expanded_budget);
        assert!(
            expanded.is_ok(),
            "complete native planning must admit exact empty-case assignments: {}",
            expanded.as_ref().err().map_or_else(String::new, |error| format!("{error:#}"))
        );
        let expanded = expanded?;
        let expanded_outputs =
            expanded.iter().map(|chunk| completed(chunk, 100)).collect::<Result<Vec<_>>>()?;
        let terminal = merge_outputs(&expanded_ids, &expanded_plans, &expanded, expanded_outputs)?;
        assert_eq!(terminal.len(), partition_count);
        assert_eq!(
            terminal.iter().filter(|output| output.is_empty()).count(),
            expanded_plans.iter().filter(|plan| plan.expected_case_count == 0).count()
        );
        for (output, plan) in terminal.iter().zip(&expanded_plans) {
            assert_eq!(output.is_empty(), plan.expected_case_count == 0);
            if output.is_empty() {
                assert_eq!(output.exit_code(), None);
                assert_eq!(output.events(), None);
                assert!(output.stderr().is_empty());
            }
        }
        let coverage = CoverageAttestation::reconcile_terminals(
            &inventory,
            terminal
                .iter()
                .zip(&expanded_plans)
                .map(|(output, plan)| (output.label(), output.events(), plan)),
        );
        assert!(coverage.complete && coverage.selected_shards_valid);
        assert!(BinaryDiscoveryPlan::selected(&metadata, Some(&BTreeSet::new()), 2).is_err());
        let empty = expanded_plans
            .iter()
            .find(|plan| plan.expected_case_count == 0)
            .context("synthetic fixture must include an empty assignment")?;
        let nonempty = expanded_plans
            .iter()
            .find(|plan| plan.expected_case_count > 0)
            .context("synthetic fixture must include a nonempty assignment")?;
        assert!(
            CoverageAttestation::reconcile_terminals(&inventory, [("empty", None, empty)])
                .selected_shards_valid
        );
        assert!(
            !CoverageAttestation::reconcile_terminals(&inventory, [("nonempty", None, nonempty)])
                .selected_shards_valid
        );
        assert!(
            !CoverageAttestation::reconcile_planned(&inventory, [("empty", b"".as_slice(), empty)])
                .selected_shards_valid
        );
        assert!(
            !CoverageAttestation::reconcile_terminals(
                &inventory,
                [("empty", None, empty), ("empty", None, empty)]
            )
            .selected_shards_valid
        );
        let foreign = ExpectedTestInventory::parse(&serde_json::to_vec(&json!({
            "test-count":1,"rust-suites":{"foreign::suite":{
                "package-name":"foreign","binary-id":"foreign::suite","binary-name":"suite",
                "kind":"test","status":"listed","testcases":{"other":{
                    "ignored":false,"filter-match":{"status":"matches"}}}
            }}
        }))?)?;
        assert!(empty.verify_inventory(&foreign).is_err());
        assert!(
            !CoverageAttestation::reconcile_terminals(&foreign, [("stale-empty", None, empty)])
                .selected_shards_valid
        );
        let empty_index = expanded_plans
            .iter()
            .position(|plan| plan.expected_case_count == 0)
            .context("empty index absent")?;
        assert!(empty
            .require_partition(&inventory, &format!("hash:{}/100", (empty_index + 1) % 100 + 1))
            .is_err());
        let empty_plan = ShardDiscoveryPlan::from_inventory(
            &inventory,
            &format!("hash:{}/100", empty_index + 1),
            "0.9.100",
        )?;
        let empty_ids = vec!["only-empty".to_string()];
        let mut only_empty_budget = expanded_budget.clone();
        only_empty_budget.selected_shards = 1;
        let only_empty_plan = vec![empty_plan];
        let only_empty_chunks =
            chunks(&metadata, &empty_ids, &only_empty_plan, &only_empty_budget)?;
        assert!(only_empty_chunks.is_empty());
        let only_empty_terminal =
            merge_outputs(&empty_ids, &only_empty_plan, &only_empty_chunks, vec![])?;
        assert_eq!(only_empty_terminal.len(), 1);
        assert!(only_empty_terminal[0].is_empty() && only_empty_terminal[0].succeeded());
        assert!(merge_outputs(&[], &[], &[], vec![]).is_err());
        assert!(merge_outputs(&expanded_ids[..99], &expanded_plans, &expanded, vec![]).is_err());
        let first = expanded.first().context("nonempty chunk absent")?;
        assert!(
            !CoverageAttestation::reconcile_terminals(
                &inventory,
                [("empty", Some(completed(first, 100)?.output.stdout.as_slice()), empty)]
            )
            .selected_shards_valid
        );
        Ok(())
    }

    #[test]
    fn missing_duplicate_unknown_or_corrupt_chunk_evidence_cannot_pass_union() -> Result<()> {
        let PartitionFixture { metadata, inventory, shard_ids: ids, shard_plans: plans, budget } =
            fixture()?;
        let planned = chunks(&metadata, &ids, &plans, &budget)?;
        assert!(merge_outputs(
            &ids,
            &plans,
            &planned,
            planned[..planned.len() - 1]
                .iter()
                .map(|chunk| completed(chunk, 5))
                .collect::<Result<Vec<_>>>()?
        )
        .is_err());
        let mut duplicate =
            planned.iter().map(|chunk| completed(chunk, 5)).collect::<Result<Vec<_>>>()?;
        duplicate.push(completed(&planned[0], 5)?);
        assert!(merge_outputs(&ids, &plans, &planned, duplicate).is_err());
        let mut unknown =
            planned.iter().map(|chunk| completed(chunk, 5)).collect::<Result<Vec<_>>>()?;
        unknown[0].label = "unknown-native-chunk".into();
        assert!(merge_outputs(&ids, &plans, &planned, unknown).is_err());
        let mut failed =
            planned.iter().map(|chunk| completed(chunk, 5)).collect::<Result<Vec<_>>>()?;
        failed[0].output.status =
            std::process::Command::new("sh").args(["-c", "exit 1"]).status()?;
        let failed = merge_outputs(&ids, &plans, &planned, failed)?;
        assert!(!failed.iter().all(ShardTerminal::succeeded));
        let mut corrupt =
            planned.iter().map(|chunk| completed(chunk, 5)).collect::<Result<Vec<_>>>()?;
        corrupt[0].output.stdout.clear();
        let merged = merge_outputs(&ids, &plans, &planned, corrupt)?;
        assert!(
            !CoverageAttestation::reconcile_terminals(
                &inventory,
                merged.iter().zip(&plans).map(|(output, plan)| (
                    output.label(),
                    output.events(),
                    plan
                ))
            )
            .complete
        );
        Ok(())
    }

    #[test]
    fn absent_producer_binary_or_unfit_native_case_demand_is_rejected() -> Result<()> {
        let PartitionFixture { metadata, shard_ids: ids, shard_plans: plans, budget, .. } =
            fixture()?;
        let mut absent: serde_json::Value = serde_json::from_slice(&metadata)?;
        absent["rust-binaries"]
            .as_object_mut()
            .context("fixture binary map")?
            .remove("owner::binary0");
        assert!(chunks(&serde_json::to_vec(&absent)?, &ids, &plans, &budget).is_err());
        let mut unfit = budget.clone();
        unfit.test_threads_per_process = 9;
        assert!(chunks(&metadata, &ids, &plans, &unfit).is_err());
        unfit = budget.clone();
        unfit.worker_budget = 1;
        assert!(chunks(&metadata, &ids, &plans, &unfit).is_err());
        Ok(())
    }
}
