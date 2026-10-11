use super::{execute_sample, PendingSampleRun};
use crate::ports::CampaignExecution;
use anyhow::{anyhow, Result};
use bijux_dna_bench_corpus::run::BenchmarkCorpusSampleRun;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
pub(crate) fn execute_pending_samples(
    program: &Path,
    repo_root: &Path,
    pending: Vec<PendingSampleRun>,
    executor: &dyn CampaignExecution,
    sample_jobs: usize,
) -> Result<Vec<BenchmarkCorpusSampleRun>> {
    if sample_jobs <= 1 || pending.len() <= 1 {
        return pending
            .into_iter()
            .map(|row| execute_sample(program, repo_root, row, executor))
            .collect();
    }

    let worker_count = sample_jobs.min(pending.len());
    let queue = Arc::new(Mutex::new(VecDeque::from(pending)));
    let results = Arc::new(Mutex::new(Vec::<BenchmarkCorpusSampleRun>::new()));

    std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::new();
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let program = program.to_path_buf();
            let repo_root = repo_root.to_path_buf();
            handles.push(scope.spawn(move || -> Result<()> {
                loop {
                    let next = {
                        let mut locked =
                            queue.lock().map_err(|_| anyhow!("benchmark work queue poisoned"))?;
                        locked.pop_front()
                    };
                    let Some(row) = next else {
                        break;
                    };
                    let run = execute_sample(&program, &repo_root, row, executor)?;
                    let mut locked =
                        results.lock().map_err(|_| anyhow!("benchmark result queue poisoned"))?;
                    locked.push(run);
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle.join().map_err(|_| anyhow!("corpus benchmark worker panicked"))??;
        }
        Ok(())
    })?;

    let mut locked = results.lock().map_err(|_| anyhow!("benchmark result queue poisoned"))?;
    Ok(std::mem::take(&mut *locked))
}
