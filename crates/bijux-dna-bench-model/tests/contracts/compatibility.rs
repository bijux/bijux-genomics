use bijux_dna_bench_model::{
    migrate_benchmark_attempt_v1, parse_benchmark_attempt_document, parse_benchmark_cycle_document,
    BenchmarkAttemptDocument, BenchmarkAttemptRecord, BenchmarkAttemptV1MigrationContext,
    BenchmarkCycleDocument, BenchmarkCycleStatus, BenchmarkEvidenceReference,
    BenchmarkJobReference, BenchmarkLifecycleStatus, BenchmarkSchedulerAccounting,
    BenchmarkSchedulerTerminalState,
};

fn legacy_attempt(status: BenchmarkLifecycleStatus) -> BenchmarkAttemptRecord {
    BenchmarkAttemptRecord::v1(
        "all-domains-20260806".to_string(),
        "run-2042".to_string(),
        "attempt-001".to_string(),
        "fastq.profile_read_lengths".to_string(),
        "seqkit".to_string(),
        status,
    )
}

fn evidence() -> BenchmarkEvidenceReference {
    BenchmarkEvidenceReference {
        uri: "study://results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit/evidence.json"
            .to_string(),
        sha256: "a".repeat(64),
        size_bytes: 42,
    }
}

fn accounting() -> BenchmarkSchedulerAccounting {
    BenchmarkSchedulerAccounting {
        scheduler: "slurm".to_string(),
        job_id: "3461001".to_string(),
        state: BenchmarkSchedulerTerminalState::Completed,
        raw_state: "COMPLETED".to_string(),
        exit_code: 0,
        signal: 0,
        elapsed_seconds: 12,
        total_cpu_seconds: 10,
        max_rss_bytes: Some(1024),
        node_list: "n001".to_string(),
        reason: None,
    }
}

#[test]
fn compatibility_readers_discriminate_and_validate_versions() -> anyhow::Result<()> {
    let v1 = legacy_attempt(BenchmarkLifecycleStatus::Planned);
    let parsed = parse_benchmark_attempt_document(&serde_json::to_vec(&v1)?)?;
    assert_eq!(parsed, BenchmarkAttemptDocument::V1(v1.clone()));

    let migrated = migrate_benchmark_attempt_v1(
        &v1,
        &BenchmarkAttemptV1MigrationContext {
            observed_at_utc: "2026-08-06T12:00:00Z".to_string(),
            accounting: Vec::new(),
            evidence: Vec::new(),
            terminal_reason: None,
        },
    )?;
    let parsed = parse_benchmark_attempt_document(&serde_json::to_vec(&migrated)?)?;
    assert_eq!(parsed, BenchmarkAttemptDocument::V2(migrated));

    let cycle = BenchmarkCycleStatus::v1(
        "all-domains-20260806".to_string(),
        "2026-08-06T12:00:00Z".to_string(),
        vec![v1],
    );
    let parsed = parse_benchmark_cycle_document(&serde_json::to_vec(&cycle)?)?;
    assert_eq!(parsed, BenchmarkCycleDocument::V1(cycle));
    Ok(())
}

#[test]
fn compatibility_readers_reject_unknown_versions_fields_and_states() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(legacy_attempt(BenchmarkLifecycleStatus::Planned))?;
    value["schema_version"] = serde_json::json!("bijux.benchmark.attempt.v99");
    assert!(parse_benchmark_attempt_document(&serde_json::to_vec(&value)?).is_err());

    let mut value = serde_json::to_value(legacy_attempt(BenchmarkLifecycleStatus::Planned))?;
    value["unexpected"] = serde_json::json!(true);
    assert!(parse_benchmark_attempt_document(&serde_json::to_vec(&value)?).is_err());

    let mut value = serde_json::to_value(legacy_attempt(BenchmarkLifecycleStatus::Planned))?;
    value["status"] = serde_json::json!("invented");
    assert!(parse_benchmark_attempt_document(&serde_json::to_vec(&value)?).is_err());
    Ok(())
}

#[test]
fn terminal_migration_requires_and_preserves_provenance() -> anyhow::Result<()> {
    let mut legacy = legacy_attempt(BenchmarkLifecycleStatus::Passed);
    legacy.jobs.push(BenchmarkJobReference {
        scheduler: "slurm".to_string(),
        job_id: "3461001".to_string(),
    });
    legacy.evidence_paths.push(evidence().uri.clone());

    let missing = BenchmarkAttemptV1MigrationContext {
        observed_at_utc: "2026-08-06T12:00:00Z".to_string(),
        accounting: Vec::new(),
        evidence: Vec::new(),
        terminal_reason: None,
    };
    assert!(migrate_benchmark_attempt_v1(&legacy, &missing).is_err());

    let migrated = migrate_benchmark_attempt_v1(
        &legacy,
        &BenchmarkAttemptV1MigrationContext {
            observed_at_utc: "2026-08-06T12:00:00Z".to_string(),
            accounting: vec![accounting()],
            evidence: vec![evidence()],
            terminal_reason: None,
        },
    )?;
    assert_eq!(migrated.current_status(), Some(BenchmarkLifecycleStatus::Passed));
    let retained =
        migrated.transitions.iter().flat_map(|transition| &transition.evidence).collect::<Vec<_>>();
    assert_eq!(retained, vec![&evidence()]);
    Ok(())
}

#[test]
fn migration_rejects_unrepresentable_legacy_terminal_state() {
    let legacy = legacy_attempt(BenchmarkLifecycleStatus::Failed);
    let result = migrate_benchmark_attempt_v1(
        &legacy,
        &BenchmarkAttemptV1MigrationContext {
            observed_at_utc: "2026-08-06T12:00:00Z".to_string(),
            accounting: Vec::new(),
            evidence: Vec::new(),
            terminal_reason: Some("historical failure reason".to_string()),
        },
    );
    assert!(result.is_err());
}

#[test]
fn nonterminal_migration_preserves_existing_evidence_paths() -> anyhow::Result<()> {
    let mut legacy = legacy_attempt(BenchmarkLifecycleStatus::Running);
    legacy.jobs.push(BenchmarkJobReference {
        scheduler: "slurm".to_string(),
        job_id: "3461001".to_string(),
    });
    legacy.evidence_paths.push(evidence().uri.clone());
    let migrated = migrate_benchmark_attempt_v1(
        &legacy,
        &BenchmarkAttemptV1MigrationContext {
            observed_at_utc: "2026-08-06T12:00:00Z".to_string(),
            accounting: Vec::new(),
            evidence: vec![evidence()],
            terminal_reason: None,
        },
    )?;
    assert_eq!(migrated.current_status(), Some(BenchmarkLifecycleStatus::Running));
    assert_eq!(migrated.transitions.last().map(|row| &row.evidence), Some(&vec![evidence()]));
    Ok(())
}
