use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use bijux_environment::api::RunnerKind;
use bijux_exec::primitives::{execute_stage_plan as execute_plan, StageResultV1};

pub(super) fn bench_jobs(requested: u32) -> usize {
    usize::try_from(requested).unwrap_or(1).clamp(1, 32)
}

pub(super) fn execute_plans_with_jobs(
    plans: Vec<bijux_core::StagePlanV1>,
    runner: RunnerKind,
    jobs: usize,
) -> Result<Vec<StageResultV1>> {
    if jobs <= 1 || plans.len() <= 1 {
        return plans
            .iter()
            .map(|plan| execute_plan(plan, runner, None))
            .collect();
    }
    let total = plans.len();
    let queue = Arc::new(Mutex::new(VecDeque::from(
        plans.into_iter().enumerate().collect::<Vec<_>>(),
    )));
    let results: Arc<Mutex<Vec<Option<Result<StageResultV1>>>>> =
        Arc::new(Mutex::new(Vec::with_capacity(total)));
    {
        let mut guard = results
            .lock()
            .map_err(|_| anyhow!("results lock poisoned"))?;
        guard.resize_with(total, || None);
    }
    let mut workers = Vec::new();
    let job_count = jobs.min(total);
    for _ in 0..job_count {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        workers.push(std::thread::spawn(move || loop {
            let next = {
                match queue.lock() {
                    Ok(mut guard) => guard.pop_front(),
                    Err(_) => None,
                }
            };
            let Some((idx, plan)) = next else {
                break;
            };
            let value = execute_plan(&plan, runner, None);
            if let Ok(mut guard) = results.lock() {
                guard[idx] = Some(value);
            } else {
                break;
            }
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }
    let results = {
        let mut guard = results
            .lock()
            .map_err(|_| anyhow!("results lock poisoned"))?;
        std::mem::take(&mut *guard)
    };
    let mut out = Vec::with_capacity(results.len());
    for entry in results {
        let value = entry.unwrap_or_else(|| Err(anyhow!("execution result missing")))?;
        out.push(value);
    }
    Ok(out)
}
