use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::runtime::crate_gate_inputs::CrateGateProofInputs;
use crate::runtime::process::ParallelProcessOutput;

use super::{
    CompiledWorkspaceExecutables, Context, FullShard, FullShardRoots, NextestReuseInputs, Result,
    ShardDiscoveryPlan, TrackedSourceSnapshot, Workspace,
};

mod native_chunks;

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

/// Local producers may reduce configured Cargo parallelism, but cannot expand its authority.
pub(super) fn admit_compiler_jobs(
    configured: usize,
    native_workers: usize,
    local_products: bool,
    control: Option<&str>,
) -> Result<usize> {
    let jobs = if local_products {
        positive_decimal("CARGO_BUILD_JOBS", control)?.unwrap_or(configured)
    } else {
        configured
    };
    anyhow::ensure!(
        (1..=16).contains(&configured) && jobs > 0 && jobs <= configured,
        "compiler jobs exceed configured producer authority"
    );
    anyhow::ensure!(
        !local_products || jobs <= native_workers,
        "compiler jobs exceed admitted local native worker allocation"
    );
    Ok(jobs)
}

pub(super) fn resolve_compiler_jobs(
    receipt: &mut serde_json::Value,
    configured: usize,
    native_workers: usize,
    local_products: bool,
) -> Result<usize> {
    receipt["compiler_admission"] =
        serde_json::json!({"state":"admission_failed", "configured_jobs":configured});
    let control = if local_products { optional_environment("CARGO_BUILD_JOBS")? } else { None };
    let jobs = admit_compiler_jobs(configured, native_workers, local_products, control.as_deref())?;
    receipt["compiler_admission"] = serde_json::json!({"state":"admitted", "configured_jobs":configured,
        "effective_jobs":jobs,"origin":if control.is_some() {"caller_local_products"} else {"configured_authority"}});
    receipt["archive_build_jobs"] = serde_json::json!(jobs);
    Ok(jobs)
}

/// The pinned executor admits two native list children for each binary and hardware-wide zstd.
pub(super) fn admit_native_transport(
    receipt: &mut serde_json::Value,
    budget: &ExecutionBudget,
    local_products: bool,
) -> Result<()> {
    let compression_workers = if local_products { 0 } else { budget.detected_cpu_capacity };
    let admitted = budget.worker_budget >= 2 && compression_workers <= budget.worker_budget;
    receipt["native_worker_admission"] = serde_json::json!({
        "state":if admitted {"admitted"} else {"blocked"}, "native_worker_budget":budget.worker_budget,
        "minimum_listing_children_per_binary":2,"archive_compression_worker_demand":compression_workers,
        "transport":if local_products {"local_metadata"} else {"archive"},"executor":"cargo-nextest0.9.100",
        "scope":"native listing and declared case workers; parent runtimes/nested tools have no hard physical CPU/process cap"});
    anyhow::ensure!(admitted,"pinned native discovery/compression exceeds allocation: budget {}, compression {}, minimum native listing 2",budget.worker_budget,compression_workers);
    Ok(())
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
    pub inventory: &'a crate::application::ci::ExpectedTestInventory,
    pub snapshot: &'a TrackedSourceSnapshot,
    pub scratch_root: &'a Path,
    pub report_root: &'a Path,
    pub reuse: &'a NextestReuseInputs,
    pub executables: &'a CompiledWorkspaceExecutables,
    pub compiled_target: &'a Path,
    pub crate_gate_inputs: &'a CrateGateProofInputs,
}

pub(super) struct ShardExecution {
    pub workspaces: Vec<PathBuf>,
    pub outputs: Vec<ParallelProcessOutput>,
    pub preparation_milliseconds: BTreeMap<String, u64>,
    pub crate_gate_proof_inputs: BTreeMap<String, serde_json::Value>,
}

pub(super) fn run_shards(
    inputs: ShardInputs<'_>,
    budget: &ExecutionBudget,
) -> Result<ShardExecution> {
    native_chunks::run_shards(inputs, budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn local_compiler_control_refuses_invalid_and_over_authority_before_production() -> Result<()> {
        assert_eq!(admit_compiler_jobs(8, 8, true, None)?, 8);
        assert_eq!(admit_compiler_jobs(8, 4, true, Some("2"))?, 2);
        assert_eq!(admit_compiler_jobs(8, 4, false, Some("invalid"))?, 8);
        for control in [
            Some(""),
            Some("0"),
            Some("+2"),
            Some(" 2"),
            Some("5"),
            Some("9"),
            Some("99999999999999999999999999999999"),
            None,
        ] {
            assert!(admit_compiler_jobs(8, 4, true, control).is_err(), "{control:?}");
        }
        assert!(admit_compiler_jobs(17, 32, true, Some("2")).is_err());
        let owner = tempfile::tempdir()?;
        let path = owner.path().join("compiler-receipt.json");
        std::fs::write(&path, serde_json::to_vec(&serde_json::json!({"all_passed":true}))?)?;
        let mut producers = 0;
        let refused = crate::runtime::gate_receipt::run_reported::<(), _>(
            &path,
            serde_json::json!({"all_passed":false}),
            Instant::now(),
            &mut producers,
            |count, _| {
                admit_compiler_jobs(8, 4, true, Some("5"))?;
                *count += 1;
                Ok(())
            },
            |_, _| Ok(()),
        );
        assert!(refused.is_err());
        assert_eq!(producers, 0);
        let terminal: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(terminal["all_passed"], false);
        assert_eq!(terminal["state"], "complete");
        assert!(terminal["execution_error"]
            .as_str()
            .context("compiler admission error absent")?
            .contains("compiler jobs exceed admitted local native worker allocation"));
        Ok(())
    }

    #[test]
    fn pinned_archive_compression_and_minimum_native_listing_refuse_before_production() -> Result<()>
    {
        let budget = ExecutionBudget::admit(5, 10, Some("8"), Some("4"), Some("2"))?;
        let mut receipt = serde_json::json!({"all_passed":false});
        assert!(admit_native_transport(&mut receipt, &budget, false).is_err());
        assert_eq!(receipt["native_worker_admission"]["archive_compression_worker_demand"], 10);
        assert_eq!(receipt["native_worker_admission"]["state"], "blocked");
        let owner = tempfile::tempdir()?;
        let path = owner.path().join("transport-receipt.json");
        std::fs::write(&path, serde_json::to_vec(&serde_json::json!({"all_passed":true}))?)?;
        let mut producers = 0;
        let refused = crate::runtime::gate_receipt::run_reported::<(), _>(
            &path,
            serde_json::json!({"all_passed":false}),
            Instant::now(),
            &mut producers,
            |count, receipt| {
                admit_native_transport(receipt, &budget, false)?;
                *count += 1;
                Ok(())
            },
            |_, _| Ok(()),
        );
        assert!(refused.is_err());
        assert_eq!(producers, 0);
        let terminal: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(terminal["all_passed"], false);
        assert_eq!(terminal["native_worker_admission"]["state"], "blocked");
        admit_native_transport(&mut receipt, &budget, true)?;
        assert_eq!(receipt["native_worker_admission"]["state"], "admitted");
        let minimum = ExecutionBudget::admit(5, 10, Some("1"), None, None)?;
        assert!(admit_native_transport(&mut receipt, &minimum, true).is_err());
        Ok(())
    }

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
