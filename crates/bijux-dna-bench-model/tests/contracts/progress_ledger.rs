use bijux_dna_bench_model::{
    validate_benchmark_progress_ledger, BenchmarkProgressAttempt, BenchmarkProgressAttemptStatus,
    BenchmarkProgressCheck, BenchmarkProgressCheckStatus, BenchmarkProgressCommit,
    BenchmarkProgressExecutionUnit, BenchmarkProgressExecutionUnitStatus,
    BenchmarkProgressExternalWait, BenchmarkProgressFrontier, BenchmarkProgressLedger,
    BenchmarkProgressLedgerSource, BenchmarkProgressOwner, BenchmarkProgressTask,
    BenchmarkProgressTaskStatus, BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1,
};

fn valid_ledger() -> BenchmarkProgressLedger {
    BenchmarkProgressLedger {
        schema_version: BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1.to_string(),
        cycle_id: "production-evidence-cycle".to_string(),
        source: BenchmarkProgressLedgerSource {
            local_head: "a".repeat(40),
            remote_head: "a".repeat(40),
            catalog_sha256: "b".repeat(64),
            benchmark_registry_sha256: "c".repeat(64),
            local_clean: Some(true),
            remote_clean: Some(true),
        },
        frontier: BenchmarkProgressFrontier {
            ready_task_ids: vec!["BENCH-001".to_string()],
            ready_unit_ids: vec!["run-2001:case-1".to_string()],
            submitted_unit_ids: Vec::new(),
            collectible_unit_ids: Vec::new(),
        },
        tasks: vec![BenchmarkProgressTask {
            task_id: "BENCH-001".to_string(),
            status: BenchmarkProgressTaskStatus::InProgress,
            next_action: "advance the complete ready case frontier once".to_string(),
            evidence: Vec::new(),
        }],
        execution_units: vec![BenchmarkProgressExecutionUnit {
            unit_id: "run-2001:case-1".to_string(),
            owner: BenchmarkProgressOwner::Benchmarks,
            run_id: "run-2001".to_string(),
            status: BenchmarkProgressExecutionUnitStatus::Ready,
            prerequisites: Vec::new(),
            attempt_ids: Vec::new(),
        }],
        attempts: Vec::new(),
        commits: vec![BenchmarkProgressCommit {
            sha: "d".repeat(40),
            intent: "persist the benchmark progress contract".to_string(),
            checks: vec!["bench-model-contracts".to_string()],
        }],
        checks: vec![BenchmarkProgressCheck {
            id: "bench-model-contracts".to_string(),
            status: BenchmarkProgressCheckStatus::Passed,
            log: "study://checks/bench-model-contracts.log".to_string(),
            duration_seconds: Some(1.5),
        }],
        external_waits: vec![BenchmarkProgressExternalWait {
            id: "licensed-artifacts".to_string(),
            reason: "four licensed artifacts require lawful site access".to_string(),
            independent_work_continues: true,
            next_observation_command: Some("bijux-dna-dev containers cycle status".to_string()),
        }],
    }
}

#[test]
fn progress_ledger_accepts_separate_task_and_execution_frontiers() {
    assert!(validate_benchmark_progress_ledger(&valid_ledger()).is_ok());
}

#[test]
fn progress_ledger_rejects_frontier_status_drift() {
    let mut ledger = valid_ledger();
    ledger.execution_units[0].status = BenchmarkProgressExecutionUnitStatus::Passed;
    assert!(validate_benchmark_progress_ledger(&ledger).is_err());
}

#[test]
fn progress_ledger_rejects_scheduler_active_attempt_without_job_identity() {
    let mut ledger = valid_ledger();
    ledger.attempts.push(BenchmarkProgressAttempt {
        attempt_id: "attempt-0001".to_string(),
        run_id: "run-2001".to_string(),
        source_commit: "a".repeat(40),
        slurm_job_id: None,
        status: BenchmarkProgressAttemptStatus::Running,
        evidence_root: None,
        failure_class: None,
    });
    assert!(validate_benchmark_progress_ledger(&ledger).is_err());
}

#[test]
fn progress_ledger_rejects_verified_task_without_evidence() {
    let mut ledger = valid_ledger();
    ledger.tasks[0].status = BenchmarkProgressTaskStatus::Verified;
    ledger.frontier.ready_task_ids.clear();
    assert!(validate_benchmark_progress_ledger(&ledger).is_err());
}

#[test]
fn progress_ledger_rejects_a_unit_in_multiple_frontier_partitions() {
    let mut ledger = valid_ledger();
    ledger.frontier.collectible_unit_ids = ledger.frontier.ready_unit_ids.clone();
    assert!(validate_benchmark_progress_ledger(&ledger).is_err());
}
