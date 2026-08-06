use bijux_dna_bench_model::{
    BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2, BenchmarkAttemptTransition,
    BenchmarkEvidenceReference, BenchmarkJobReference, BenchmarkLifecycleStatus,
    BenchmarkRunIdentity, BenchmarkSchedulerAccounting, BenchmarkSchedulerEvidenceSource,
    BenchmarkSchedulerObservationDraft, BenchmarkSchedulerObservedState,
    BenchmarkSchedulerTerminalState,
};
use bijux_dna_runtime::{
    append_attempt_lifecycle_transition, benchmark_attempt_directory, ensure_attempt_lifecycle,
    initialize_attempt_lifecycle, load_attempt_lifecycle, load_attempt_scheduler_observations,
    record_attempt_acceptance, record_attempt_accounting, record_attempt_blocked_external,
    record_attempt_ready, record_attempt_running, record_attempt_scheduler_observation,
    record_attempt_submission, record_attempt_superseded,
};

fn identity(attempt_id: &str) -> BenchmarkAttemptIdentity {
    BenchmarkAttemptIdentity {
        campaign_id: "all-domains-20260804".to_string(),
        run: BenchmarkRunIdentity {
            run_id: "run-2042".to_string(),
            domain: "fastq".to_string(),
            stage_id: "fastq.profile_read_lengths".to_string(),
        },
        attempt_id: attempt_id.to_string(),
        tool_id: "seqkit".to_string(),
    }
}

fn planned(attempt_id: &str) -> BenchmarkAttemptRecordV2 {
    BenchmarkAttemptRecordV2::planned(identity(attempt_id), "2026-08-04T08:00:00Z".to_string())
}

fn ready_transition() -> BenchmarkAttemptTransition {
    BenchmarkAttemptTransition {
        sequence: 2,
        from_status: Some(BenchmarkLifecycleStatus::Planned),
        to_status: BenchmarkLifecycleStatus::Ready,
        observed_at_utc: "2026-08-04T08:01:00Z".to_string(),
        reason: None,
        jobs: Vec::new(),
        accounting: Vec::new(),
        evidence: Vec::new(),
    }
}

fn blocked_transition() -> BenchmarkAttemptTransition {
    BenchmarkAttemptTransition {
        sequence: 3,
        from_status: Some(BenchmarkLifecycleStatus::Ready),
        to_status: BenchmarkLifecycleStatus::BlockedExternal,
        observed_at_utc: "2026-08-04T08:02:00Z".to_string(),
        reason: Some("required reference panel is unavailable".to_string()),
        jobs: Vec::new(),
        accounting: Vec::new(),
        evidence: Vec::new(),
    }
}

fn job() -> BenchmarkJobReference {
    BenchmarkJobReference { scheduler: "slurm".to_string(), job_id: "3461001".to_string() }
}

fn accounting(state: BenchmarkSchedulerTerminalState) -> BenchmarkSchedulerAccounting {
    let (raw_state, reason) = match state {
        BenchmarkSchedulerTerminalState::Completed => ("COMPLETED", None),
        BenchmarkSchedulerTerminalState::Cancelled => {
            ("CANCELLED", Some("operator requested scheduler cancellation".to_string()))
        }
        _ => ("TIMEOUT", Some("scheduler wall-time limit was exceeded".to_string())),
    };
    BenchmarkSchedulerAccounting {
        scheduler: "slurm".to_string(),
        job_id: "3461001".to_string(),
        state,
        raw_state: raw_state.to_string(),
        exit_code: i32::from(state != BenchmarkSchedulerTerminalState::Completed),
        signal: 0,
        elapsed_seconds: 125,
        total_cpu_seconds: 90,
        max_rss_bytes: Some(2 * 1024 * 1024),
        node_list: "n001".to_string(),
        reason,
    }
}

fn running_observation(job_id: &str, observed_at_utc: &str) -> BenchmarkSchedulerObservationDraft {
    BenchmarkSchedulerObservationDraft {
        scheduler: "slurm".to_string(),
        job_id: job_id.to_string(),
        state: BenchmarkSchedulerObservedState::Running,
        source: BenchmarkSchedulerEvidenceSource::Queue,
        observed_at_utc: observed_at_utc.to_string(),
        accounting: None,
    }
}

#[test]
fn scheduler_observation_journal_retains_repeated_running_queries() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-scheduler-observation-journal");
    let initial = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &initial)?;
    record_attempt_ready(temporary.path(), &initial.identity, "2026-08-04T08:01:00Z")?;
    record_attempt_submission(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:02:00Z",
        vec![job()],
    )?;

    let first = record_attempt_scheduler_observation(
        temporary.path(),
        &initial.identity,
        running_observation("3461001", "2026-08-04T08:03:00Z"),
    )?;
    record_attempt_running(temporary.path(), &initial.identity, "2026-08-04T08:03:00Z", &job())?;
    let second = record_attempt_scheduler_observation(
        temporary.path(),
        &initial.identity,
        running_observation("3461001", "2026-08-04T08:04:00Z"),
    )?;

    assert_eq!((first.sequence, second.sequence), (1, 2));
    assert_eq!(load_attempt_scheduler_observations(temporary.path(), &initial.identity)?.len(), 2);
    assert_eq!(load_attempt_lifecycle(temporary.path(), &initial.identity)?.transitions.len(), 4);
    assert!(record_attempt_scheduler_observation(
        temporary.path(),
        &initial.identity,
        running_observation("3461001", "2026-08-04T08:04:00Z"),
    )
    .is_err());
    assert!(record_attempt_scheduler_observation(
        temporary.path(),
        &initial.identity,
        running_observation("3461002", "2026-08-04T08:05:00Z"),
    )
    .is_err());
    Ok(())
}

fn evidence() -> BenchmarkEvidenceReference {
    BenchmarkEvidenceReference {
        uri: "study://results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit/evidence.json"
            .to_string(),
        sha256: "d".repeat(64),
        size_bytes: 42,
    }
}

#[test]
fn lifecycle_store_uses_permanent_run_tool_campaign_and_attempt_identity() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-identity");
    let record = planned("attempt-001");

    let directory = initialize_attempt_lifecycle(temporary.path(), &record)?;

    assert_eq!(
        directory.strip_prefix(temporary.path())?,
        std::path::Path::new(
            "run-2042-fastq-profile-read-lengths/seqkit/attempts/all-domains-20260804/attempt-001"
        )
    );
    assert_eq!(load_attempt_lifecycle(temporary.path(), &record.identity)?, record);
    Ok(())
}

#[test]
fn lifecycle_store_appends_sequence_addressed_immutable_events() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-append");
    let initial = planned("attempt-001");
    let directory = initialize_attempt_lifecycle(temporary.path(), &initial)?;

    let record = append_attempt_lifecycle_transition(
        temporary.path(),
        &initial.identity,
        ready_transition(),
    )?;

    assert_eq!(record.current_status(), Some(BenchmarkLifecycleStatus::Ready));
    assert!(directory.join("transitions/00000002.json").is_file());
    assert_eq!(load_attempt_lifecycle(temporary.path(), &initial.identity)?, record);
    Ok(())
}

#[test]
fn lifecycle_store_refuses_identity_reuse_and_illegal_events() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-refusal");
    let initial = planned("attempt-001");
    let directory = initialize_attempt_lifecycle(temporary.path(), &initial)?;
    let initial_bytes = std::fs::read(directory.join("attempt.json"))?;

    assert!(initialize_attempt_lifecycle(temporary.path(), &initial).is_err());
    let illegal = BenchmarkAttemptTransition {
        to_status: BenchmarkLifecycleStatus::Running,
        ..ready_transition()
    };
    assert!(
        append_attempt_lifecycle_transition(temporary.path(), &initial.identity, illegal).is_err()
    );
    assert!(!directory.join("transitions").exists());
    assert_eq!(std::fs::read(directory.join("attempt.json"))?, initial_bytes);
    Ok(())
}

#[test]
fn lifecycle_store_recovers_only_an_identical_planned_initialization() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-recovery");
    let initial = planned("attempt-001");
    let first_directory = ensure_attempt_lifecycle(temporary.path(), &initial)?;
    let initial_bytes = std::fs::read(first_directory.join("attempt.json"))?;

    let recovered_directory = ensure_attempt_lifecycle(temporary.path(), &initial)?;
    assert_eq!(first_directory, recovered_directory);
    assert_eq!(std::fs::read(recovered_directory.join("attempt.json"))?, initial_bytes);

    let mut conflicting = initial.clone();
    conflicting.transitions[0].observed_at_utc = "2026-08-04T08:00:01Z".to_string();
    assert!(ensure_attempt_lifecycle(temporary.path(), &conflicting).is_err());
    assert_eq!(std::fs::read(first_directory.join("attempt.json"))?, initial_bytes);
    Ok(())
}

#[test]
fn lifecycle_store_retains_multiple_attempt_ids_for_one_tool() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-multiple");
    let first = planned("attempt-001");
    let second = planned("attempt-002");

    initialize_attempt_lifecycle(temporary.path(), &first)?;
    initialize_attempt_lifecycle(temporary.path(), &second)?;

    assert_ne!(
        benchmark_attempt_directory(temporary.path(), &first.identity)?,
        benchmark_attempt_directory(temporary.path(), &second.identity)?
    );
    assert_eq!(load_attempt_lifecycle(temporary.path(), &first.identity)?, first);
    assert_eq!(load_attempt_lifecycle(temporary.path(), &second.identity)?, second);
    Ok(())
}

#[test]
fn lifecycle_store_requires_existing_terminal_retry_lineage() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-lineage");
    let prior = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &prior)?;
    append_attempt_lifecycle_transition(temporary.path(), &prior.identity, ready_transition())?;

    let mut premature_retry = planned("attempt-002");
    premature_retry.supersedes_attempt_id = Some("attempt-001".to_string());
    assert!(initialize_attempt_lifecycle(temporary.path(), &premature_retry).is_err());

    append_attempt_lifecycle_transition(temporary.path(), &prior.identity, blocked_transition())?;
    initialize_attempt_lifecycle(temporary.path(), &premature_retry)?;

    let mut unknown_retry = planned("attempt-003");
    unknown_retry.supersedes_attempt_id = Some("attempt-999".to_string());
    assert!(initialize_attempt_lifecycle(temporary.path(), &unknown_retry).is_err());
    Ok(())
}

#[test]
fn lifecycle_store_ignores_foreign_artifacts_and_rejects_malformed_evidence() -> anyhow::Result<()>
{
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-malformed");
    let initial = planned("attempt-001");
    let directory = initialize_attempt_lifecycle(temporary.path(), &initial)?;
    std::fs::write(directory.join("unexpected.txt"), b"not governed")?;
    assert_eq!(load_attempt_lifecycle(temporary.path(), &initial.identity)?, initial);

    std::fs::create_dir(directory.join("transitions"))?;
    std::fs::write(directory.join("transitions/2.json"), b"{}")?;
    assert!(load_attempt_lifecycle(temporary.path(), &initial.identity).is_err());
    Ok(())
}

#[test]
fn lifecycle_store_serializes_concurrent_transition_publication() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-concurrent");
    let initial = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &initial)?;
    let root = std::sync::Arc::new(temporary.path().to_path_buf());

    let handles = (0..2)
        .map(|_| {
            let root = std::sync::Arc::clone(&root);
            let identity = initial.identity.clone();
            std::thread::spawn(move || {
                append_attempt_lifecycle_transition(&root, &identity, ready_transition()).is_ok()
            })
        })
        .collect::<Vec<_>>();
    let outcomes =
        handles.into_iter().map(|handle| handle.join().unwrap_or(false)).collect::<Vec<_>>();

    assert_eq!(outcomes.iter().filter(|outcome| **outcome).count(), 1);
    assert_eq!(
        load_attempt_lifecycle(temporary.path(), &initial.identity)?.current_status(),
        Some(BenchmarkLifecycleStatus::Ready)
    );
    Ok(())
}

#[test]
fn evidence_operations_admit_a_fast_completed_scheduler_job_idempotently() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-evidence-success");
    let initial = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &initial)?;
    record_attempt_ready(temporary.path(), &initial.identity, "2026-08-04T08:01:00Z")?;
    record_attempt_submission(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:02:00Z",
        vec![job()],
    )?;

    let terminal = accounting(BenchmarkSchedulerTerminalState::Completed);
    let collecting = record_attempt_accounting(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:03:00Z",
        terminal.clone(),
    )?;
    let repeated = record_attempt_accounting(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:04:00Z",
        terminal,
    )?;
    assert_eq!(collecting, repeated);
    assert_eq!(collecting.current_status(), Some(BenchmarkLifecycleStatus::Collecting));

    let passed = record_attempt_acceptance(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:05:00Z",
        vec![evidence()],
    )?;
    let repeated = record_attempt_acceptance(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:06:00Z",
        vec![evidence()],
    )?;
    assert_eq!(passed, repeated);
    assert_eq!(passed.current_status(), Some(BenchmarkLifecycleStatus::Passed));
    Ok(())
}

#[test]
fn evidence_operations_retain_running_and_failed_scheduler_evidence() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-evidence-failure");
    let initial = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &initial)?;
    record_attempt_ready(temporary.path(), &initial.identity, "2026-08-04T08:01:00Z")?;
    record_attempt_submission(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:02:00Z",
        vec![job()],
    )?;
    record_attempt_running(temporary.path(), &initial.identity, "2026-08-04T08:03:00Z", &job())?;

    let failed = record_attempt_accounting(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:04:00Z",
        accounting(BenchmarkSchedulerTerminalState::Timeout),
    )?;
    assert_eq!(failed.current_status(), Some(BenchmarkLifecycleStatus::Failed));
    assert_eq!(
        failed.transitions.last().and_then(|event| event.reason.as_deref()),
        Some("scheduler wall-time limit was exceeded")
    );
    assert_eq!(failed.transitions.last().map(|event| event.accounting.len()), Some(1));
    Ok(())
}

#[test]
fn evidence_operations_retain_cancelled_scheduler_evidence() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-evidence-cancelled");
    let initial = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &initial)?;
    record_attempt_ready(temporary.path(), &initial.identity, "2026-08-04T08:01:00Z")?;
    record_attempt_submission(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:02:00Z",
        vec![job()],
    )?;

    let accounting = accounting(BenchmarkSchedulerTerminalState::Cancelled);
    let cancelled = record_attempt_accounting(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:03:00Z",
        accounting.clone(),
    )?;
    let repeated = record_attempt_accounting(
        temporary.path(),
        &initial.identity,
        "2026-08-04T08:04:00Z",
        accounting,
    )?;

    assert_eq!(cancelled, repeated);
    assert_eq!(cancelled.current_status(), Some(BenchmarkLifecycleStatus::Cancelled));
    assert_eq!(
        cancelled.transitions.last().and_then(|event| event.reason.as_deref()),
        Some("operator requested scheduler cancellation")
    );
    Ok(())
}

#[test]
fn evidence_operations_record_external_and_supersession_decisions_idempotently(
) -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-attempt-evidence-decisions");
    let blocked = planned("attempt-001");
    initialize_attempt_lifecycle(temporary.path(), &blocked)?;
    let first = record_attempt_blocked_external(
        temporary.path(),
        &blocked.identity,
        "2026-08-04T08:01:00Z",
        "required reference panel is unavailable",
    )?;
    let repeated = record_attempt_blocked_external(
        temporary.path(),
        &blocked.identity,
        "2026-08-04T08:02:00Z",
        "required reference panel is unavailable",
    )?;
    assert_eq!(first, repeated);

    let superseded = planned("attempt-002");
    initialize_attempt_lifecycle(temporary.path(), &superseded)?;
    let record = record_attempt_superseded(
        temporary.path(),
        &superseded.identity,
        "2026-08-04T08:01:00Z",
        "campaign contract was replaced before submission",
    )?;
    assert_eq!(record.current_status(), Some(BenchmarkLifecycleStatus::Superseded));
    Ok(())
}
