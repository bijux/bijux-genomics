fn pair_counts_from_paths(
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<(Option<u64>, Option<u64>)> {
    let pairs_in = if inputs.len() >= 2 {
        let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
        Some(r1.reads.min(r2.reads))
    } else {
        None
    };
    let pairs_out = if outputs.len() >= 2 {
        let r1 = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(outputs.get(1).map(PathBuf::as_path))?;
        Some(r1.reads.min(r2.reads))
    } else {
        None
    };
    Ok((pairs_in, pairs_out))
}

fn stats_or_zero(path: Option<&Path>) -> Result<bijux_core::measure::SeqkitMetrics> {
    if let Some(path) = path {
        if path.exists() {
            if path.is_dir() {
                return Ok(bijux_core::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0 {
                return Ok(bijux_core::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            return fastq_stats(path);
        }
    }
    Ok(bijux_core::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    })
}

fn zero_seqkit_metrics() -> bijux_core::measure::SeqkitMetrics {
    bijux_core::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    }
}

fn observer_jobs() -> usize {
    std::env::var("BIJUX_OBSERVER_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map_or(2, |value| value.clamp(1, 32))
}

fn stats_for_paths(paths: &[Option<&Path>]) -> Result<Vec<bijux_core::measure::SeqkitMetrics>> {
    let tasks: Vec<(usize, Option<PathBuf>)> = paths
        .iter()
        .enumerate()
        .map(|(idx, path)| (idx, path.map(Path::to_path_buf)))
        .collect();
    if tasks.len() <= 1 || observer_jobs() == 1 {
        return tasks
            .into_iter()
            .map(|(_, path)| stats_or_zero(path.as_deref()))
            .collect();
    }
    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let mut initial = Vec::with_capacity(paths.len());
    initial.resize_with(paths.len(), || None);
    let results: Arc<Mutex<Vec<Option<Result<bijux_core::measure::SeqkitMetrics>>>>> =
        Arc::new(Mutex::new(initial));
    let mut workers = Vec::new();
    let job_count = observer_jobs().min(paths.len());
    for _ in 0..job_count {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        workers.push(std::thread::spawn(move || loop {
            let next = {
                match queue.lock() {
                    Ok(mut queue) => queue.pop_front(),
                    Err(_) => None,
                }
            };
            let Some((idx, path)) = next else {
                break;
            };
            let value = stats_or_zero(path.as_deref());
            if let Ok(mut results) = results.lock() {
                results[idx] = Some(value);
            }
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }
    let results = Arc::try_unwrap(results)
        .map_err(|_| anyhow!("observer results still shared"))?
        .into_inner()
        .unwrap_or_default();
    let mut out = Vec::with_capacity(results.len());
    for entry in results {
        let value = entry.unwrap_or_else(|| Err(anyhow!("observer result missing")))?;
        out.push(value);
    }
    Ok(out)
}

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
}

fn observer_result_from_plan(
    plan: &StagePlanV1,
    outputs: Vec<PathBuf>,
    exit_code: i32,
    stdout: String,
    stderr: String,
) -> bijux_engine::core::types::StageResult {
    bijux_engine::core::types::StageResult {
        invocation: bijux_engine::core::types::ToolInvocation {
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            inputs: plan
                .io
                .inputs
                .iter()
                .map(|artifact| artifact.path.clone())
                .collect(),
            params: plan.params.clone(),
            requirements: None,
        },
        exit_code,
        stdout,
        stderr,
        outputs,
    }
}

#[derive(Debug, Clone, Copy)]
struct RetentionCounts {
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_u64(value: u64) -> f64 {
    value as f64
}
