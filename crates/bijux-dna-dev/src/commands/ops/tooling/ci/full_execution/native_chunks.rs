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

use super::super::{full_shard_args, full_shard_env, materialize_full_shard_workspace};
use super::{ExecutionBudget, ShardExecution, ShardInputs};

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
    let groups = plans
        .iter()
        .map(|plan| {
            BinaryDiscoveryPlan::selected(
                metadata,
                Some(&plan.selected_binary_ids()),
                per_command_budget,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let mut chunks = Vec::new();
    let longest = groups.iter().map(|plan| plan.groups.len()).max().unwrap_or_default();
    let mut predecessors = BTreeMap::new();
    // Interleave ready partitions; terminal edges prevent same-workspace groups from overlapping.
    for index in 0..longest {
        for (shard_index, plan) in groups.iter().enumerate() {
            if let Some(group) = plan.groups.get(index) {
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
    chunks: &[NativeChunk],
    outputs: Vec<ParallelProcessOutput>,
) -> Result<Vec<ParallelProcessOutput>> {
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
            Ok(ParallelProcessOutput {
                label: label.clone(),
                duration,
                output: Output {
                    status: status.context("native chunk verdict absent")?,
                    stdout,
                    stderr,
                },
            })
        })
        .collect()
}

pub(super) fn run_shards(
    inputs: ShardInputs<'_>,
    budget: &ExecutionBudget,
) -> Result<ShardExecution> {
    ensure!(
        inputs.shards.len() == inputs.roots.len() && inputs.shards.len() == inputs.plans.len(),
        "full-shard execution inputs differ from admitted roster"
    );
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
    let receipt_path = inputs.report_root.join("native-execution-chunks.receipt.json");
    let mut receipt = json!({"schema_version":"bijux.ci.bounded_native_execution.v1","all_passed":false,
        "source_commit":inputs.snapshot.commit(),"source_patch_sha256":inputs.snapshot.patch_sha256(),
        "producer_binary_metadata_sha256":producer_sha256,"native_worker_budget":budget.worker_budget,
        "worker_scope":"ordinary/ignored native listing and declared case workers; parent runtimes and nested tools have no hard physical CPU/process cap",
        "metadata_inputs":metadata_inputs,
        "expected_chunks":plan.iter().map(|chunk|json!({"label":chunk.label,"shard":shard_ids[chunk.shard_index],"binaries":chunk.binaries,"resource_units":chunk.resource_units,"terminal_dependencies":chunk.terminal_dependencies})).collect::<Vec<_>>()});
    bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
    let preparation = Mutex::new(BTreeMap::new());
    let proof_handles = Mutex::new(BTreeMap::new());
    let outputs = ProcessRunner::new(inputs.workspace).run_prepared_weighted_parallel_logged(
        specifications,
        budget.process_jobs,
        budget.worker_budget,
        inputs.report_root,
        |admitted| {
            let chunk = plan
                .iter()
                .find(|chunk| chunk.label == admitted.label)
                .context("admitted native chunk absent")?;
            let index = chunk.shard_index;
            let started = Instant::now();
            if chunk.terminal_dependencies.is_empty() {
                let workspace = materialize_full_shard_workspace(
                    inputs.snapshot,
                    inputs.scratch_root,
                    &shard_ids[index],
                )?;
                ensure!(
                    workspace == workspaces[index],
                    "native chunk preparation changed workspace authority"
                );
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
        },
    );
    let handles = proof_handles
        .into_inner()
        .map_err(|_| anyhow::anyhow!("native proof receipt lock poisoned"))?;
    let mut crate_gate_proof_inputs = BTreeMap::new();
    let mut failures = Vec::new();
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
    let merged = outputs.and_then(|outputs| merge_outputs(&shard_ids, &plan, outputs));
    if let Ok(outputs) = &merged {
        let coverage = CoverageAttestation::reconcile_planned(
            inputs.inventory,
            outputs.iter().zip(inputs.plans).map(|(output, plan)| {
                (output.label.as_str(), output.output.stdout.as_slice(), plan)
            }),
        );
        receipt["all_passed"] = json!(
            failures.is_empty()
                && coverage.selected_shards_valid
                && outputs.iter().all(|output| output.output.status.success())
        );
        receipt["coverage"] = json!(coverage);
    }
    receipt["aggregation_error"] = json!(merged.as_ref().err().map(|error| format!("{error:#}")));
    bijux_dna_infra::write_bytes(&receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
    ensure!(failures.is_empty(), "native chunk physical inputs changed: {}", failures.join("; "));
    let merged = merged?;
    for output in &merged {
        bijux_dna_infra::write_bytes(
            inputs.report_root.join(format!("{}.stdout.log", output.label)),
            &output.output.stdout,
        )?;
        bijux_dna_infra::write_bytes(
            inputs.report_root.join(format!("{}.stderr.log", output.label)),
            &output.output.stderr,
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

    fn completed(chunk: &NativeChunk) -> Result<ParallelProcessOutput> {
        let metadata: serde_json::Value = serde_json::from_slice(&chunk.metadata)?;
        let mut records = Vec::new();
        for binary in metadata["rust-binaries"].as_object().context("fixture binaries")?.values() {
            let name = binary["binary-name"].as_str().context("fixture name")?;
            let cases = (0..32)
                .map(|case| format!("case-{case}"))
                .filter(|name| {
                    XxHash64::oneshot(0, name.as_bytes()) % 5 == chunk.shard_index as u64
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
        let mut observed = planned.iter().map(completed).collect::<Result<Vec<_>>>()?;
        observed.reverse();
        let merged = merge_outputs(&ids, &planned, observed)?;
        let coverage = CoverageAttestation::reconcile_planned(
            &inventory,
            merged.iter().zip(&plans).map(|(output, plan)| {
                (output.label.as_str(), output.output.stdout.as_slice(), plan)
            }),
        );
        assert!(coverage.complete && coverage.selected_shards_valid);
        assert_eq!(merged.len(), 5);
        assert!(inventory.ignored_case_count() > 0);
        Ok(())
    }

    #[test]
    fn missing_duplicate_unknown_or_corrupt_chunk_evidence_cannot_pass_union() -> Result<()> {
        let PartitionFixture { metadata, inventory, shard_ids: ids, shard_plans: plans, budget } =
            fixture()?;
        let planned = chunks(&metadata, &ids, &plans, &budget)?;
        assert!(merge_outputs(
            &ids,
            &planned,
            planned[..planned.len() - 1].iter().map(completed).collect::<Result<Vec<_>>>()?
        )
        .is_err());
        let mut duplicate = planned.iter().map(completed).collect::<Result<Vec<_>>>()?;
        duplicate.push(completed(&planned[0])?);
        assert!(merge_outputs(&ids, &planned, duplicate).is_err());
        let mut unknown = planned.iter().map(completed).collect::<Result<Vec<_>>>()?;
        unknown[0].label = "unknown-native-chunk".into();
        assert!(merge_outputs(&ids, &planned, unknown).is_err());
        let mut corrupt = planned.iter().map(completed).collect::<Result<Vec<_>>>()?;
        corrupt[0].output.stdout.clear();
        let merged = merge_outputs(&ids, &planned, corrupt)?;
        assert!(
            !CoverageAttestation::reconcile_planned(
                &inventory,
                merged.iter().zip(&plans).map(|(output, plan)| (
                    output.label.as_str(),
                    output.output.stdout.as_slice(),
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
