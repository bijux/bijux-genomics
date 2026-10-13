use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use crate::runtime::process::ParallelProcessOutput;

use super::{
    full_shard_args, full_shard_env, materialize_full_shard_workspace,
    CompiledWorkspaceExecutables, Context, FullShard, FullShardRoots, NextestReuseInputs,
    ParallelProcessSpec, ProcessRunner, Result, ShardDiscoveryPlan, TrackedSourceSnapshot,
    Workspace,
};

const WORKER_BUDGET_ENV: &str = "BIJUX_CI_FULL_WORKER_BUDGET";
const SHARD_JOBS_ENV: &str = "BIJUX_CI_FULL_SHARD_JOBS";
const TEST_THREADS_ENV: &str = "BIJUX_CI_FULL_SHARD_TEST_THREADS";
const DEFAULT_TEST_THREADS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ExecutionBudget {
    pub detected_cpu_capacity: usize,
    pub worker_budget: usize,
    pub allocation_origin: &'static str,
    pub selected_shards: usize,
    pub process_jobs: usize,
    pub test_threads_per_process: usize,
    pub maximum_case_workers: usize,
    pub preparation_jobs: usize,
}

impl ExecutionBudget {
    pub(super) fn resolve(selected_shards: usize) -> Result<Self> {
        let cpu_capacity =
            std::thread::available_parallelism().context("detect full-shard CPU allocation")?.get();
        let allocation = optional_environment(WORKER_BUDGET_ENV)?;
        let jobs = optional_environment(SHARD_JOBS_ENV)?;
        let threads = optional_environment(TEST_THREADS_ENV)?;
        Self::admit(
            selected_shards,
            cpu_capacity,
            allocation.as_deref(),
            jobs.as_deref(),
            threads.as_deref(),
        )
    }

    fn admit(
        selected: usize,
        detected: usize,
        allocation: Option<&str>,
        jobs: Option<&str>,
        threads: Option<&str>,
    ) -> Result<Self> {
        anyhow::ensure!(
            selected > 0 && detected > 0,
            "full-shard roster and CPU capacity must be positive"
        );
        let worker_budget = positive_decimal(WORKER_BUDGET_ENV, allocation)?.unwrap_or(detected);
        let declared_threads = positive_decimal(TEST_THREADS_ENV, threads)?;
        let default_jobs = (worker_budget / declared_threads.unwrap_or(1)).max(1).min(selected);
        let process_jobs =
            positive_decimal(SHARD_JOBS_ENV, jobs)?.unwrap_or(default_jobs).min(selected);
        let test_threads_per_process = declared_threads
            .unwrap_or_else(|| (worker_budget / process_jobs).clamp(1, DEFAULT_TEST_THREADS));
        let maximum_case_workers = process_jobs
            .checked_mul(test_threads_per_process)
            .context("full-shard case-worker product overflow")?;
        anyhow::ensure!(maximum_case_workers <= worker_budget,
            "full-shard process jobs ({process_jobs}) times case threads ({test_threads_per_process}) exceeds allocation ({worker_budget})");
        Ok(Self {
            detected_cpu_capacity: detected,
            worker_budget,
            allocation_origin: if allocation.is_some() {
                "explicit_worker_budget"
            } else {
                "detected_cpu_capacity"
            },
            selected_shards: selected,
            process_jobs,
            test_threads_per_process,
            maximum_case_workers,
            preparation_jobs: process_jobs,
        })
    }
}

pub(super) fn admit_budget(
    receipt: &mut serde_json::Value,
    resolve: impl FnOnce() -> Result<ExecutionBudget>,
) -> Result<ExecutionBudget> {
    match resolve() {
        Ok(budget) => {
            receipt["execution_resources"] = serde_json::to_value(&budget)?;
            Ok(budget)
        }
        Err(error) => {
            receipt["execution_resources"] = serde_json::json!({"state": "admission_failed"});
            Err(error)
        }
    }
}

fn optional_environment(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error).with_context(|| format!("read {name}")),
    }
}

fn positive_decimal(name: &str, value: Option<&str>) -> Result<Option<usize>> {
    value
        .map(|value| {
            anyhow::ensure!(
                !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()),
                "{name} must be a positive decimal integer"
            );
            let value = value.parse::<usize>().with_context(|| format!("parse {name}"))?;
            anyhow::ensure!(value > 0, "{name} must be positive");
            Ok(value)
        })
        .transpose()
}

pub(super) struct ShardInputs<'a> {
    pub workspace: &'a Workspace,
    pub shards: &'a [FullShard],
    pub roots: &'a [FullShardRoots],
    pub plans: &'a [ShardDiscoveryPlan],
    pub snapshot: &'a TrackedSourceSnapshot,
    pub scratch_root: &'a Path,
    pub report_root: &'a Path,
    pub reuse: &'a NextestReuseInputs,
    pub executables: &'a CompiledWorkspaceExecutables,
    pub compiled_target: &'a Path,
}

pub(super) struct ShardExecution {
    pub workspaces: Vec<PathBuf>,
    pub outputs: Vec<ParallelProcessOutput>,
    pub preparation_milliseconds: BTreeMap<String, u64>,
}

pub(super) fn run_shards(
    inputs: ShardInputs<'_>,
    budget: &ExecutionBudget,
) -> Result<ShardExecution> {
    anyhow::ensure!(
        inputs.shards.len() == inputs.roots.len()
            && inputs.shards.len() == inputs.plans.len()
            && inputs.shards.len() == budget.selected_shards,
        "full-shard execution inputs differ from admitted roster"
    );
    let labels = inputs.shards.iter().map(|shard| shard.id.clone()).collect::<Vec<_>>();
    let indices = labels
        .iter()
        .enumerate()
        .map(|(index, label)| (label.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let workspaces = labels
        .iter()
        .map(|label| inputs.scratch_root.join("workspaces").join(label))
        .collect::<Vec<_>>();
    let preparation = Mutex::new(BTreeMap::new());
    let outputs = ProcessRunner::new(inputs.workspace).run_prepared_parallel_logged(
        labels.clone(),
        budget.process_jobs,
        inputs.report_root,
        |label| {
            let index = indices[label];
            let started = Instant::now();
            let shard = &inputs.shards[index];
            let workspace =
                materialize_full_shard_workspace(inputs.snapshot, inputs.scratch_root, label)?;
            anyhow::ensure!(
                workspace == workspaces[index],
                "full-shard materialization changed its admitted workspace"
            );
            let mut args =
                full_shard_args(shard, inputs.reuse, &workspace, budget.test_threads_per_process)?;
            args.extend(["--filterset".to_string(), inputs.plans[index].filterset.clone()]);
            let mut envs = full_shard_env(&inputs.roots[index], &workspace)?;
            envs.extend(inputs.executables.remapped_environment(inputs.compiled_target)?);
            preparation
                .lock()
                .map_err(|_| anyhow::anyhow!("full-shard preparation receipt lock poisoned"))?
                .insert(label.to_string(), super::duration_milliseconds(started.elapsed()));
            Ok(ParallelProcessSpec {
                label: label.to_string(),
                program: "cargo".to_string(),
                args,
                envs,
                current_dir: Some(workspace),
            })
        },
    )?;
    let preparation_milliseconds = preparation
        .into_inner()
        .map_err(|_| anyhow::anyhow!("full-shard preparation receipt lock poisoned"))?;
    Ok(ShardExecution { workspaces, outputs, preparation_milliseconds })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hundreds_of_partitions_share_a_small_admitted_worker_allocation() -> Result<()> {
        for selected in [100, 256] {
            let budget = ExecutionBudget::admit(selected, 6, None, None, None)?;
            assert_eq!(budget.process_jobs, 6);
            assert_eq!(budget.preparation_jobs, 6);
            assert_eq!(budget.test_threads_per_process, 1);
            assert_eq!(budget.maximum_case_workers, 6);
        }
        let budget = ExecutionBudget::admit(5, 32, None, None, None)?;
        assert_eq!(
            (budget.process_jobs, budget.test_threads_per_process, budget.maximum_case_workers),
            (5, 6, 30)
        );
        Ok(())
    }

    #[test]
    fn explicit_allocation_is_independent_of_detected_capacity_and_selected_roster() -> Result<()> {
        let budget = ExecutionBudget::admit(256, 8, Some("400"), Some("100"), Some("4"))?;
        assert_eq!((budget.process_jobs, budget.maximum_case_workers), (100, 400));
        assert_eq!(budget.detected_cpu_capacity, 8);
        assert_eq!(budget.allocation_origin, "explicit_worker_budget");
        let budget = ExecutionBudget::admit(2, 8, None, Some("100"), Some("4"))?;
        assert_eq!((budget.process_jobs, budget.maximum_case_workers), (2, 8));
        Ok(())
    }

    #[test]
    fn threads_override_reduces_default_jobs_to_fit_the_allocation() -> Result<()> {
        let budget = ExecutionBudget::admit(100, 16, None, None, Some("8"))?;
        assert_eq!((budget.process_jobs, budget.maximum_case_workers), (2, 16));
        let budget = ExecutionBudget::admit(256, 1, None, None, None)?;
        assert_eq!((budget.process_jobs, budget.test_threads_per_process), (1, 1));
        Ok(())
    }

    #[test]
    fn invalid_and_over_budget_admission_is_rejected() {
        for value in
            ["", "0", "-1", "+1", " 2", "2 ", "1.5", "one", "9999999999999999999999999999999"]
        {
            for overrides in
                [[Some(value), None, None], [None, Some(value), None], [None, None, Some(value)]]
            {
                assert!(ExecutionBudget::admit(100, 6, overrides[0], overrides[1], overrides[2])
                    .is_err());
            }
        }
        assert!(ExecutionBudget::admit(100, 6, None, Some("4"), Some("2")).is_err());
        assert!(ExecutionBudget::admit(100, 6, None, None, Some("7")).is_err());
        assert!(ExecutionBudget::admit(0, 6, None, None, None).is_err());
        assert!(ExecutionBudget::admit(100, 0, None, None, None).is_err());
        assert!(ExecutionBudget::admit(
            2,
            1,
            Some(&usize::MAX.to_string()),
            Some("2"),
            Some(&usize::MAX.to_string())
        )
        .is_err());
    }

    #[test]
    fn invalid_budget_replaces_prior_green_with_a_terminal_failure_before_production() -> Result<()>
    {
        use crate::runtime::gate_receipt::run_reported;
        use serde_json::json;

        let root = tempfile::tempdir()?;
        let path = root.path().join("full-gate.json");
        std::fs::write(
            &path,
            serde_json::to_vec(&json!({"state": "complete", "all_passed": true}))?,
        )?;
        let mut producers = 0;
        let result = run_reported::<(), _>(
            &path,
            json!({"source_commit": "candidate-source", "all_passed": false,
                "execution_resources": {"state": "pending_admission"}}),
            Instant::now(),
            &mut producers,
            |count, receipt| {
                admit_budget(receipt, || ExecutionBudget::admit(100, 6, Some("0"), None, None))?;
                *count += 1;
                Ok(())
            },
            |_, _| Ok(()),
        );
        assert!(result.is_err());
        assert_eq!(producers, 0);
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(receipt["state"], "complete");
        assert_eq!(receipt["all_passed"], false);
        assert_eq!(receipt["source_commit"], "candidate-source");
        assert_eq!(receipt["execution_resources"]["state"], "admission_failed");
        assert!(receipt["execution_error"]
            .as_str()
            .context("budget failure absent")?
            .contains(WORKER_BUDGET_ENV));
        Ok(())
    }
}
