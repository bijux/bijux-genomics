use crate::internal::fastq::stages::preprocess::*;

use anyhow::anyhow;

#[allow(clippy::too_many_lines)]
pub(super) fn execute_preprocess_batch(
    batch: &[ExecutionStep],
    runner: RuntimeKind,
    jobs: usize,
    out_dir: &std::path::Path,
    normalized_sample_id: &str,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> Result<Vec<StageResultV1>> {
    let mut resumable = Vec::new();
    let mut pending = Vec::new();
    for (idx, planned) in batch.iter().enumerate() {
        let stage_id = planned.step_id.to_string();
        let stage_root = run_artifacts_dir_for_out(out_dir).join(planned.step_id.as_str());
        write_stage_path_contract(&stage_root, &stage_id, planned, args.r2.is_some())?;
        let expected_outputs =
            planned.io.outputs.iter().map(|artifact| artifact.path.clone()).collect::<Vec<_>>();
        let runtime_marker = stage_root.join("runtime_provenance.json");
        let resume_hit =
            runtime_marker.exists() && expected_outputs.iter().all(|path| path.exists());
        if resume_hit {
            resumable.push((
                idx,
                StageResultV1 {
                    run_id: format!("fastq-preprocess-{}", planned.step_id),
                    exit_code: 0,
                    runtime_s: 0.0,
                    memory_mb: 0.0,
                    outputs: expected_outputs,
                    metrics_path: None,
                    stdout: "resumed".to_string(),
                    stderr: String::new(),
                    command: "resume".to_string(),
                },
            ));
            continue;
        }
        pending.push((
            idx,
            execution_kernel::ToolInvocationRequest {
                step: planned.clone(),
                runner,
                context: execution_kernel::ToolContext {
                    run_id: format!("fastq-preprocess-{}", planned.step_id),
                    stage_id: planned.step_id.to_string(),
                    tool_id: planned.image.image.clone(),
                    sample_id: Some(normalized_sample_id.to_string()),
                    stage_root: stage_root.clone(),
                    input_root: args
                        .r1
                        .parent()
                        .map_or_else(|| out_dir.to_path_buf(), std::path::Path::to_path_buf),
                    output_root: out_dir.to_path_buf(),
                    tmp_root: stage_root.join("tmp"),
                    threads: 1,
                    memory_hint_mb: None,
                    compression_threads: Some(1),
                    seed: None,
                    network_policy: stage_network_policy(&stage_id),
                },
                timeout: None,
                mode: execution_kernel::ToolExecMode::Execute,
            },
        ));
    }
    let executed = if jobs <= 1 || pending.len() <= 1 {
        pending
            .iter()
            .map(|(_, request)| {
                execution_kernel::ToolExec::invoke(request).map(|result| result.stage_result)
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        let total = pending.len();
        let queue = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::from(
            pending.iter().map(|(_, request)| request.clone()).enumerate().collect::<Vec<_>>(),
        )));
        let results: std::sync::Arc<std::sync::Mutex<Vec<Option<Result<StageResultV1>>>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(total)));
        {
            let mut guard =
                results.lock().map_err(|_| anyhow!("preprocess batch results lock poisoned"))?;
            guard.resize_with(total, || None);
        }
        let job_count = jobs.min(total);
        let mut workers = Vec::new();
        for _ in 0..job_count {
            let queue = std::sync::Arc::clone(&queue);
            let results = std::sync::Arc::clone(&results);
            workers.push(std::thread::spawn(move || loop {
                let next = {
                    match queue.lock() {
                        Ok(mut guard) => guard.pop_front(),
                        Err(_) => None,
                    }
                };
                let Some((slot, request)) = next else {
                    break;
                };
                let value =
                    execution_kernel::ToolExec::invoke(&request).map(|result| result.stage_result);
                if let Ok(mut guard) = results.lock() {
                    guard[slot] = Some(value);
                } else {
                    break;
                }
            }));
        }
        for worker in workers {
            let _ = worker.join();
        }
        let results = {
            let mut guard =
                results.lock().map_err(|_| anyhow!("preprocess batch results lock poisoned"))?;
            std::mem::take(&mut *guard)
        };
        let mut out = Vec::with_capacity(results.len());
        for entry in results {
            let value = entry
                .unwrap_or_else(|| Err(anyhow!("preprocess batch execution result missing")))?;
            out.push(value);
        }
        out
    };
    let mut results = vec![None; batch.len()];
    for (idx, result) in resumable {
        results[idx] = Some(result);
    }
    for ((idx, _), result) in pending.into_iter().zip(executed) {
        results[idx] = Some(result);
    }
    results
        .into_iter()
        .map(|result| result.ok_or_else(|| anyhow!("missing batch execution result")))
        .collect()
}

#[cfg(all(test, unix))]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use bijux_dna_core::contract::{Objective, StageIO};
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageId,
        StepId, ToolConstraints,
    };
    use bijux_dna_planner_fastq::stage_api::args::{BenchFastqPreprocessArgs, FastqPlannerMode};

    fn batch_args(root: &std::path::Path) -> BenchFastqPreprocessArgs {
        BenchFastqPreprocessArgs {
            sample_id: "sample".to_string(),
            profile: None,
            r1: root.join("input.fastq"),
            r2: None,
            reference_fasta: None,
            out: root.to_path_buf(),
            strict: false,
            auto: false,
            objective: Objective::Balanced,
            bench_corpus: None,
            allow_partial: false,
            dry_run: false,
            replicates: 1,
            jobs: 2,
            ci_bootstrap: None,
            adapter_bank_preset: None,
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            enable_contaminant_removal: false,
            no_qc_post: true,
            force_merge: false,
            enable_correct: false,
            run_all_governed_tools: false,
            allow_planned: false,
            mode: FastqPlannerMode::Shotgun,
        }
    }

    fn local_step(root: &std::path::Path, label: &str) -> ExecutionStep {
        let out_dir = root.join(label);
        std::fs::create_dir_all(&out_dir).expect("create local step directory");
        let output = out_dir.join("observed.txt");
        ExecutionStep {
            step_id: StepId::new(format!("fastq.trim_reads.{label}")),
            stage_id: StageId::from_static("fastq.trim_reads"),
            command: CommandSpecV1 {
                template: vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    "printf '%s' \"$1\" > \"$2\"; printf '%s' \"$1\"".to_string(),
                    "batch-output".to_string(),
                    label.to_string(),
                    output.display().to_string(),
                ],
            },
            image: ContainerImageRefV1 {
                image: "local-shell".to_string(),
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: Vec::new(),
                outputs: vec![ArtifactSpec::required(
                    ArtifactId::new(label),
                    output,
                    ArtifactRole::Log,
                )],
            },
            out_dir,
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        }
    }

    #[test]
    fn mixed_resumed_and_pending_preprocess_batch_preserves_ordered_native_outputs() -> Result<()> {
        for jobs in [1, 2] {
            let temp = tempfile::tempdir()?;
            let root = temp.path();
            let args = batch_args(root);
            std::fs::write(&args.r1, b"@read\nACGT\n+\nIIII\n")?;
            let batch =
                [local_step(root, "retained"), local_step(root, "left"), local_step(root, "right")];
            let retained = &batch[0].io.outputs[0].path;
            std::fs::write(retained, b"retained original output")?;
            let stage_root = run_artifacts_dir_for_out(root).join(batch[0].step_id.as_str());
            std::fs::create_dir_all(&stage_root)?;
            std::fs::write(stage_root.join("runtime_provenance.json"), b"{}")?;

            let results =
                execute_preprocess_batch(&batch, RuntimeKind::Local, jobs, root, "sample", &args)?;
            assert_eq!(results.len(), 3);
            assert_eq!(results[0].command, "resume");
            assert_eq!(std::fs::read(retained)?, b"retained original output");
            for (slot, label) in [(1, "left"), (2, "right")] {
                assert_eq!(results[slot].exit_code, 0);
                assert_eq!(results[slot].stdout, label);
                assert_eq!(results[slot].outputs, vec![batch[slot].io.outputs[0].path.clone()]);
                assert_eq!(std::fs::read_to_string(&batch[slot].io.outputs[0].path)?, label);
            }
        }
        Ok(())
    }
}
