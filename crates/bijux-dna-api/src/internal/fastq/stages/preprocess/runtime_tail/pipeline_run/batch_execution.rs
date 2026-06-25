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
            pending.clone(),
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
