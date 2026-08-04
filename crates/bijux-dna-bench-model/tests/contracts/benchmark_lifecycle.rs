use bijux_dna_bench_model::{
    append_attempt_transition, benchmark_run_directory_name, validate_attempt_archive,
    validate_attempt_record_v2, validate_cycle_status_v2, BenchmarkArchiveFileRecord,
    BenchmarkArchiveSourcePaths, BenchmarkAttemptArchive, BenchmarkAttemptDisposition,
    BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2, BenchmarkAttemptTransition,
    BenchmarkCycleStatusV2, BenchmarkJobReference, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
    BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
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
        evidence_paths: Vec::new(),
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
            evidence_paths: vec![
                "study://results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit/evidence.json"
                    .to_string(),
            ],
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
    no_evidence.transitions[4].evidence_paths.clear();
    assert!(validate_attempt_record_v2(&no_evidence).is_err());

    let mut unsafe_evidence = valid.clone();
    unsafe_evidence.transitions[4].evidence_paths[0] = "study://results/../secret".to_string();
    assert!(validate_attempt_record_v2(&unsafe_evidence).is_err());
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
