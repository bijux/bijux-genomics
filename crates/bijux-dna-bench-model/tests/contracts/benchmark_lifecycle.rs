use bijux_dna_bench_model::{
    append_attempt_transition, benchmark_run_directory_name, validate_attempt_archive,
    validate_attempt_record_v2, validate_cycle_status_v2, validate_scheduler_observation,
    BenchmarkArchiveFileRecord, BenchmarkArchiveSourcePaths, BenchmarkAttemptArchive,
    BenchmarkAttemptDisposition, BenchmarkAttemptIdentity, BenchmarkAttemptMigrationState,
    BenchmarkAttemptMigrationVerification, BenchmarkAttemptMigrationVerificationEntry,
    BenchmarkAttemptRecordV2, BenchmarkAttemptTransition, BenchmarkCampaignPlan,
    BenchmarkCycleStatusV2, BenchmarkEvidenceReference, BenchmarkJobReference,
    BenchmarkLifecycleStatus, BenchmarkRunIdentity, BenchmarkSchedulerAccounting,
    BenchmarkSchedulerEvidenceSource, BenchmarkSchedulerObservation,
    BenchmarkSchedulerObservationDraft, BenchmarkSchedulerObservedState,
    BenchmarkSchedulerTerminalState, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1,
};

fn run_identity() -> BenchmarkRunIdentity {
    BenchmarkRunIdentity {
        run_id: "run-2042".to_string(),
        domain: "fastq".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
    }
}

fn attempt(attempt_id: &str, tool_id: &str) -> BenchmarkAttemptRecordV2 {
    BenchmarkAttemptRecordV2::planned(
        BenchmarkAttemptIdentity {
            campaign_id: "all-domains-20260804".to_string(),
            run: run_identity(),
            attempt_id: attempt_id.to_string(),
            tool_id: tool_id.to_string(),
        },
        "2026-08-04T08:00:00Z".to_string(),
    )
}

fn transition(
    sequence: u32,
    from_status: BenchmarkLifecycleStatus,
    to_status: BenchmarkLifecycleStatus,
    minute: u32,
) -> BenchmarkAttemptTransition {
    BenchmarkAttemptTransition {
        sequence,
        from_status: Some(from_status),
        to_status,
        observed_at_utc: format!("2026-08-04T08:{minute:02}:00Z"),
        reason: None,
        jobs: Vec::new(),
        accounting: Vec::new(),
        evidence: Vec::new(),
    }
}

fn passed_attempt(attempt_id: &str, tool_id: &str) -> BenchmarkAttemptRecordV2 {
    let mut record = attempt(attempt_id, tool_id);
    let transitions = [
        transition(
            2,
            BenchmarkLifecycleStatus::Planned,
            BenchmarkLifecycleStatus::Ready,
            1,
        ),
        BenchmarkAttemptTransition {
            jobs: vec![BenchmarkJobReference {
                scheduler: "slurm".to_string(),
                job_id: "3461001".to_string(),
            }],
            ..transition(
                3,
                BenchmarkLifecycleStatus::Ready,
                BenchmarkLifecycleStatus::Submitted,
                2,
            )
        },
        transition(
            4,
            BenchmarkLifecycleStatus::Submitted,
            BenchmarkLifecycleStatus::Running,
            3,
        ),
        BenchmarkAttemptTransition {
            accounting: vec![BenchmarkSchedulerAccounting {
                scheduler: "slurm".to_string(),
                job_id: "3461001".to_string(),
                state: BenchmarkSchedulerTerminalState::Completed,
                raw_state: "COMPLETED".to_string(),
                exit_code: 0,
                signal: 0,
                elapsed_seconds: 125,
                total_cpu_seconds: 90,
                max_rss_bytes: Some(2 * 1024 * 1024),
                node_list: "n001".to_string(),
                reason: None,
            }],
            evidence: vec![BenchmarkEvidenceReference {
                uri: "study://results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit/evidence.json"
                    .to_string(),
                sha256: "d".repeat(64),
                size_bytes: 42,
            }],
            ..transition(
                5,
                BenchmarkLifecycleStatus::Running,
                BenchmarkLifecycleStatus::Collecting,
                4,
            )
        },
        transition(
            6,
            BenchmarkLifecycleStatus::Collecting,
            BenchmarkLifecycleStatus::Passed,
            5,
        ),
    ];
    for event in transitions {
        assert!(append_attempt_transition(&mut record, event).is_ok());
    }
    record
}

fn archive(disposition: BenchmarkAttemptDisposition) -> BenchmarkAttemptArchive {
    BenchmarkAttemptArchive {
        schema_version: BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1.to_string(),
        run_id: "run-2043".to_string(),
        attempt_id: "slurm-3459974".to_string(),
        tool_id: "seqfu".to_string(),
        slurm_job_id: "3459974".to_string(),
        disposition,
        reason: None,
        archived_at_utc: "2026-08-04T00:00:00Z".to_string(),
        source_paths: BenchmarkArchiveSourcePaths {
            submission: "study://containers/evidence/submissions/run-2043-seqfu".to_string(),
            result: "study://results/benchmarks/run-2043-fastq-profile-reads/seqfu".to_string(),
        },
        evidence_sha256: "a".repeat(64),
        accounting_sha256: "b".repeat(64),
        files: vec![BenchmarkArchiveFileRecord {
            path: "result/evidence.json".to_string(),
            size_bytes: 42,
            sha256: "c".repeat(64),
        }],
    }
}

#[test]
fn failed_attempt_archive_is_identity_complete() {
    assert!(validate_attempt_archive(&archive(BenchmarkAttemptDisposition::Failed)).is_ok());
}

#[test]
fn superseded_attempt_archive_requires_reason() {
    let mut record = archive(BenchmarkAttemptDisposition::Superseded);
    let error = validate_attempt_archive(&record).err().map(|value| value.to_string());
    assert!(error.is_some_and(|value| value.contains("substantive reason")));

    record.reason = Some("measurement rows used an invalid delimiter".to_string());
    assert!(validate_attempt_archive(&record).is_ok());
}

#[test]
fn attempt_archive_rejects_malformed_digest_and_duplicate_path() {
    let mut record = archive(BenchmarkAttemptDisposition::Failed);
    record.files.push(record.files[0].clone());
    let error = validate_attempt_archive(&record).err().map(|value| value.to_string());
    assert!(error.is_some_and(|value| value.contains("duplicated")));

    record.files.pop();
    record.accounting_sha256 = "ABC".to_string();
    let error = validate_attempt_archive(&record).err().map(|value| value.to_string());
    assert!(error.is_some_and(|value| value.contains("SHA-256")));
}

#[test]
fn run_directory_name_is_derived_from_validated_identity() {
    assert_eq!(
        benchmark_run_directory_name(&run_identity()).ok().as_deref(),
        Some("run-2042-fastq-profile-read-lengths")
    );

    let mut mismatched = run_identity();
    mismatched.domain = "bam".to_string();
    assert!(benchmark_run_directory_name(&mismatched).is_err());

    let mut malformed = run_identity();
    malformed.run_id = "run-42".to_string();
    assert!(benchmark_run_directory_name(&malformed).is_err());

    let mut aliased = run_identity();
    aliased.run_id = "run-02042".to_string();
    assert!(benchmark_run_directory_name(&aliased).is_err());
}

#[test]
fn checked_lifecycle_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "attempt-v1.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_attempt_schema())?,
        ),
        (
            "attempt-v2.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_attempt_v2_schema())?,
        ),
        (
            "attempt-archive.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_attempt_archive_schema())?,
        ),
        (
            "attempt-migration-plan.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_attempt_migration_plan_schema())?,
        ),
        (
            "attempt-migration-ledger.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_attempt_migration_ledger_schema(),
            )?,
        ),
        (
            "attempt-migration-verification.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_attempt_migration_verification_schema(),
            )?,
        ),
        (
            "cycle-status-v1.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_cycle_status_schema())?,
        ),
        (
            "cycle-status-v2.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_cycle_status_v2_schema())?,
        ),
        (
            "campaign-plan.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_campaign_plan_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "lifecycle schema drifted: {name}");
    }
    Ok(())
}

#[test]
fn migration_verification_round_trips_and_rejects_schema_drift() -> anyhow::Result<()> {
    let verification = BenchmarkAttemptMigrationVerification {
        schema_version: BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1.to_string(),
        plan_id: "1".repeat(64),
        entries: vec![BenchmarkAttemptMigrationVerificationEntry {
            source: "study://containers/evidence/failures/run-2042".to_string(),
            target: "study://results/benchmark-attempts/failures/run-2042/attempt-001".to_string(),
            state: BenchmarkAttemptMigrationState::Applied,
            current_location: "study://results/benchmark-attempts/failures/run-2042/attempt-001"
                .to_string(),
            files: vec![BenchmarkArchiveFileRecord {
                path: "archive.json".to_string(),
                size_bytes: 1,
                sha256: "2".repeat(64),
            }],
        }],
    };
    let value = serde_json::to_value(&verification)?;
    assert_eq!(
        serde_json::from_value::<BenchmarkAttemptMigrationVerification>(value.clone())?,
        verification
    );
    let mut unsupported = value;
    unsupported["schema_version"] =
        serde_json::json!("bijux.benchmark.attempt_migration_verification.v2");
    assert!(serde_json::from_value::<BenchmarkAttemptMigrationVerification>(unsupported).is_err());
    Ok(())
}

#[test]
fn attempt_lifecycle_accepts_a_complete_append_only_chain() {
    let record = passed_attempt("attempt-001", "seqkit");
    assert_eq!(record.current_status(), Some(BenchmarkLifecycleStatus::Passed));
    assert!(validate_attempt_record_v2(&record).is_ok());
}

#[test]
fn append_rejects_discontinuous_illegal_and_post_terminal_events_without_mutation() {
    let mut record = attempt("attempt-001", "seqkit");
    let original = record.clone();
    let gap = transition(3, BenchmarkLifecycleStatus::Planned, BenchmarkLifecycleStatus::Ready, 1);
    assert!(append_attempt_transition(&mut record, gap).is_err());
    assert_eq!(record, original);

    let illegal =
        transition(2, BenchmarkLifecycleStatus::Planned, BenchmarkLifecycleStatus::Running, 1);
    assert!(append_attempt_transition(&mut record, illegal).is_err());
    assert_eq!(record, original);

    let mut passed = passed_attempt("attempt-002", "seqkit");
    let terminal = passed.clone();
    let after_pass =
        transition(7, BenchmarkLifecycleStatus::Passed, BenchmarkLifecycleStatus::Collecting, 6);
    assert!(append_attempt_transition(&mut passed, after_pass).is_err());
    assert_eq!(passed, terminal);
}

#[test]
fn passed_attempt_requires_slurm_and_portable_evidence() {
    let valid = passed_attempt("attempt-001", "seqkit");

    let mut no_job = valid.clone();
    no_job.transitions[2].jobs.clear();
    assert!(validate_attempt_record_v2(&no_job).is_err());

    let mut invalid_job = valid.clone();
    invalid_job.transitions[2].jobs[0].scheduler = "local".to_string();
    assert!(validate_attempt_record_v2(&invalid_job).is_err());

    let mut no_evidence = valid.clone();
    no_evidence.transitions[4].evidence.clear();
    assert!(validate_attempt_record_v2(&no_evidence).is_err());

    let mut no_accounting = valid.clone();
    no_accounting.transitions[4].accounting.clear();
    assert!(validate_attempt_record_v2(&no_accounting).is_err());

    let mut unsafe_evidence = valid.clone();
    unsafe_evidence.transitions[4].evidence[0].uri = "study://results/../secret".to_string();
    assert!(validate_attempt_record_v2(&unsafe_evidence).is_err());

    let mut unchecksummed = valid.clone();
    unchecksummed.transitions[4].evidence[0].sha256 = "unknown".to_string();
    assert!(validate_attempt_record_v2(&unchecksummed).is_err());

    let mut contradictory = valid.clone();
    contradictory.transitions[4].accounting[0].state = BenchmarkSchedulerTerminalState::Timeout;
    assert!(validate_attempt_record_v2(&contradictory).is_err());
}

#[test]
fn failed_scheduler_attempt_retains_exact_terminal_accounting() {
    let mut failed = passed_attempt("attempt-001", "seqkit");
    failed.transitions.truncate(5);
    let terminal = &mut failed.transitions[4];
    terminal.to_status = BenchmarkLifecycleStatus::Failed;
    terminal.reason = Some("scheduler wall-time limit was exceeded".to_string());
    terminal.accounting[0].state = BenchmarkSchedulerTerminalState::Timeout;
    terminal.accounting[0].raw_state = "TIMEOUT".to_string();
    terminal.accounting[0].exit_code = 1;
    terminal.accounting[0].reason = Some("scheduler wall-time limit was exceeded".to_string());

    assert!(validate_attempt_record_v2(&failed).is_ok());

    let duplicate = failed.transitions[4].accounting[0].clone();
    failed.transitions[4].accounting.push(duplicate);
    assert!(validate_attempt_record_v2(&failed).is_err());
}

#[test]
fn cancelled_scheduler_attempt_retains_distinct_terminal_accounting() {
    let mut cancelled = passed_attempt("attempt-001", "seqkit");
    cancelled.transitions.truncate(5);
    let terminal = &mut cancelled.transitions[4];
    terminal.to_status = BenchmarkLifecycleStatus::Cancelled;
    terminal.reason = Some("scheduler cancelled the benchmark job on operator request".to_string());
    terminal.accounting[0].state = BenchmarkSchedulerTerminalState::Cancelled;
    terminal.accounting[0].raw_state = "CANCELLED".to_string();
    terminal.accounting[0].exit_code = 1;
    terminal.accounting[0].reason = Some("operator requested cancellation".to_string());

    assert!(validate_attempt_record_v2(&cancelled).is_ok());

    cancelled.transitions[4].to_status = BenchmarkLifecycleStatus::Failed;
    assert!(validate_attempt_record_v2(&cancelled).is_err());
}

#[test]
fn completed_accounting_can_follow_submission_when_running_was_not_observed() {
    let mut completed = passed_attempt("attempt-001", "seqkit");
    completed.transitions.remove(3);
    completed.transitions[3].sequence = 4;
    completed.transitions[3].from_status = Some(BenchmarkLifecycleStatus::Submitted);
    completed.transitions[4].sequence = 5;

    assert!(validate_attempt_record_v2(&completed).is_ok());
}

#[test]
fn attempt_timestamps_are_utc_and_monotonic() {
    let mut non_utc = passed_attempt("attempt-001", "seqkit");
    non_utc.transitions[1].observed_at_utc = "2026-08-04T10:01:00+02:00".to_string();
    assert!(validate_attempt_record_v2(&non_utc).is_err());

    let mut reversed = passed_attempt("attempt-002", "seqkit");
    reversed.transitions[3].observed_at_utc = "2026-08-04T07:59:00Z".to_string();
    assert!(validate_attempt_record_v2(&reversed).is_err());

    let mut unsafe_campaign = passed_attempt("attempt-003", "seqkit");
    unsafe_campaign.identity.campaign_id = "../campaign".to_string();
    assert!(validate_attempt_record_v2(&unsafe_campaign).is_err());
}

#[test]
fn cycle_preserves_multiple_attempts_with_same_run_and_tool_lineage() {
    let prior = passed_attempt("attempt-001", "seqkit");
    let mut current = attempt("attempt-002", "seqkit");
    current.supersedes_attempt_id = Some("attempt-001".to_string());
    let cycle = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![prior.clone(), current.clone()],
    );
    assert!(validate_cycle_status_v2(&cycle).is_ok());

    let second_active = attempt("attempt-003", "seqkit");
    let conflicting = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![prior.clone(), current, second_active],
    );
    assert!(validate_cycle_status_v2(&conflicting).is_err());

    let mut cross_tool = attempt("attempt-004", "fastp");
    cross_tool.supersedes_attempt_id = Some("attempt-001".to_string());
    let invalid_lineage = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![prior, cross_tool],
    );
    assert!(validate_cycle_status_v2(&invalid_lineage).is_err());
}

#[test]
fn cycle_scopes_attempt_ids_by_run_and_tool_identity() {
    let seqkit = attempt("attempt-001", "seqkit");
    let seqfu = attempt("attempt-001", "seqfu");
    let cycle = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![seqkit.clone(), seqfu],
    );
    assert!(validate_cycle_status_v2(&cycle).is_ok());

    let duplicate = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![seqkit.clone(), seqkit],
    );
    assert!(validate_cycle_status_v2(&duplicate).is_err());
}

#[test]
fn cycle_resolves_supersession_within_the_same_run_and_tool() {
    let seqkit_prior = passed_attempt("attempt-001", "seqkit");
    let seqfu_prior = passed_attempt("attempt-001", "seqfu");
    let mut seqfu_retry = attempt("attempt-002", "seqfu");
    seqfu_retry.supersedes_attempt_id = Some("attempt-001".to_string());
    let cycle = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![seqkit_prior, seqfu_prior, seqfu_retry],
    );

    assert!(validate_cycle_status_v2(&cycle).is_ok());
}

#[test]
fn campaign_plan_requires_full_source_and_registry_identities() -> anyhow::Result<()> {
    let cycle = BenchmarkCycleStatusV2::new(
        "all-domains-20260804".to_string(),
        "2026-08-04T09:00:00Z".to_string(),
        vec![attempt("attempt-001", "seqkit")],
    );
    let plan = BenchmarkCampaignPlan::new("a".repeat(40), "b".repeat(64), cycle);
    assert!(bijux_dna_bench_model::validate_campaign_plan(&plan).is_ok());

    let mut abbreviated = plan.clone();
    abbreviated.repository_commit = "a123456".to_string();
    assert!(bijux_dna_bench_model::validate_campaign_plan(&abbreviated).is_err());

    let mut uppercase = plan.clone();
    uppercase.campaign_registry_sha256 = "B".repeat(64);
    assert!(bijux_dna_bench_model::validate_campaign_plan(&uppercase).is_err());

    let mut zero = plan;
    zero.repository_commit = "0".repeat(40);
    assert!(bijux_dna_bench_model::validate_campaign_plan(&zero).is_err());

    let schema = serde_json::to_value(bijux_dna_bench_model::benchmark_campaign_plan_schema())?;
    assert_eq!(
        schema
            .pointer("/$defs/BenchmarkCampaignPlan/properties/schema_version/const")
            .or_else(|| schema.pointer("/properties/schema_version/const"))
            .and_then(serde_json::Value::as_str),
        Some(bijux_dna_bench_model::BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1)
    );
    Ok(())
}

fn scheduler_accounting() -> BenchmarkSchedulerAccounting {
    BenchmarkSchedulerAccounting {
        scheduler: "slurm".to_string(),
        job_id: "3461001".to_string(),
        state: BenchmarkSchedulerTerminalState::Completed,
        raw_state: "COMPLETED".to_string(),
        exit_code: 0,
        signal: 0,
        elapsed_seconds: 125,
        total_cpu_seconds: 90,
        max_rss_bytes: Some(2 * 1024 * 1024),
        node_list: "n001".to_string(),
        reason: None,
    }
}

fn scheduler_observation(
    state: BenchmarkSchedulerObservedState,
    source: BenchmarkSchedulerEvidenceSource,
    accounting: Option<BenchmarkSchedulerAccounting>,
) -> BenchmarkSchedulerObservation {
    BenchmarkSchedulerObservation::v1(
        1,
        BenchmarkSchedulerObservationDraft {
            scheduler: "slurm".to_string(),
            job_id: "3461001".to_string(),
            state,
            source,
            observed_at_utc: "2026-08-04T08:03:00Z".to_string(),
            accounting,
        },
    )
}

#[test]
fn scheduler_observation_requires_exact_source_state_and_accounting_closure() {
    let running = scheduler_observation(
        BenchmarkSchedulerObservedState::Running,
        BenchmarkSchedulerEvidenceSource::Queue,
        None,
    );
    assert!(validate_scheduler_observation(&running).is_ok());

    let completed = scheduler_observation(
        BenchmarkSchedulerObservedState::Completed,
        BenchmarkSchedulerEvidenceSource::Accounting,
        Some(scheduler_accounting()),
    );
    assert!(validate_scheduler_observation(&completed).is_ok());

    let mut source_drift = running;
    source_drift.observation.source = BenchmarkSchedulerEvidenceSource::Accounting;
    assert!(validate_scheduler_observation(&source_drift).is_err());

    let mut job_drift = completed;
    if let Some(accounting) = job_drift.observation.accounting.as_mut() {
        accounting.job_id = "3461002".to_string();
    }
    assert!(validate_scheduler_observation(&job_drift).is_err());
}
