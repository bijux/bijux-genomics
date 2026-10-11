use bijux_dna_bench::{
    benchmark_attempt_directory, ensure_stage_campaign_attempts, load_attempt_lifecycle,
    load_checked_in_benchmark_planning_contracts,
    plan_stage_campaign_attempts as derive_campaign_plan, plan_stage_campaign_retry_attempts,
    recover_stage_campaign_retry_lineage, verify_campaign_plan_source, BenchmarkCampaignPlan,
    BenchmarkLifecycleStatus, BenchmarkStageCampaignRegistry,
};

fn plan_stage_campaign_attempts(
    registry: &BenchmarkStageCampaignRegistry,
    run_id: &str,
    campaign_id: &str,
    attempt_id: &str,
    observed_at_utc: &str,
) -> anyhow::Result<BenchmarkCampaignPlan> {
    derive_campaign_plan(
        registry,
        &"a".repeat(40),
        &"b".repeat(64),
        run_id,
        campaign_id,
        attempt_id,
        observed_at_utc,
    )
}

#[test]
fn campaign_planner_derives_one_canonical_attempt_per_declared_tool() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;

    let cycle = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;

    assert_eq!(cycle.repository_commit, "a".repeat(40));
    assert_eq!(cycle.campaign_registry_sha256, "b".repeat(64));
    assert_eq!(cycle.cycle.attempts.len(), 2);
    assert_eq!(
        cycle
            .cycle
            .attempts
            .iter()
            .map(|attempt| attempt.identity.tool_id.as_str())
            .collect::<Vec<_>>(),
        ["seqfu", "seqkit"]
    );
    assert!(cycle.cycle.attempts.iter().all(|attempt| {
        attempt.identity.attempt_id == "attempt-001"
            && attempt.identity.run.run_id == "run-2042"
            && attempt.identity.run.domain == "fastq"
            && attempt.identity.run.stage_id == "fastq.profile_read_lengths"
            && attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned)
    }));
    for attempt in &cycle.cycle.attempts {
        let path = benchmark_attempt_directory(
            std::path::Path::new("/benchmark-root"),
            &attempt.identity,
        )?;
        assert!(path.starts_with(format!(
            "/benchmark-root/run-2042-fastq-profile-read-lengths/{}/attempts/current-source-campaign/attempt-001",
            attempt.identity.tool_id
        )));
    }
    Ok(())
}

#[test]
fn campaign_planner_is_deterministic_for_identical_inputs() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let first = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    let second = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;

    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn campaign_planner_refuses_unknown_run_and_malformed_lifecycle_identity() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    assert!(plan_stage_campaign_attempts(
        &registry,
        "run-2999",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )
    .is_err());
    assert!(plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "../unsafe-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )
    .is_err());
    assert!(plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-one",
        "2026-08-05T00:00:00Z",
    )
    .is_err());
    assert!(plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T02:00:00+02:00",
    )
    .is_err());
    Ok(())
}

#[test]
fn campaign_storage_is_restart_safe_and_uses_immutable_run_directories() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let cycle = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-storage");

    let first = ensure_stage_campaign_attempts(temporary.path(), &cycle)?;
    let recovered = ensure_stage_campaign_attempts(temporary.path(), &cycle)?;

    assert_eq!(first, recovered);
    assert_eq!(first.len(), 2);
    for attempt in &cycle.cycle.attempts {
        assert_eq!(load_attempt_lifecycle(temporary.path(), &attempt.identity)?, *attempt);
    }
    Ok(())
}

#[test]
fn campaign_storage_recovers_attempts_that_advanced_from_the_same_origin() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let cycle = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-advanced-recovery");
    let initialized = ensure_stage_campaign_attempts(temporary.path(), &cycle)?;
    bijux_dna_runtime::record_attempt_ready(
        temporary.path(),
        &cycle.cycle.attempts[0].identity,
        "2026-08-05T00:01:00Z",
    )?;

    let recovered = ensure_stage_campaign_attempts(temporary.path(), &cycle)?;

    assert_eq!(recovered, initialized);
    assert_eq!(
        load_attempt_lifecycle(temporary.path(), &cycle.cycle.attempts[0].identity)?
            .current_status(),
        Some(BenchmarkLifecycleStatus::Ready)
    );
    Ok(())
}

#[test]
fn campaign_storage_preflights_all_existing_attempts_before_writing() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let cycle = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-conflict");
    let conflicting = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:01Z",
    )?;
    bijux_dna_bench::initialize_attempt_lifecycle(
        temporary.path(),
        &conflicting.cycle.attempts[1],
    )?;

    assert!(ensure_stage_campaign_attempts(temporary.path(), &cycle).is_err());
    let missing = benchmark_attempt_directory(temporary.path(), &cycle.cycle.attempts[0].identity)?;
    assert!(!missing.exists());
    Ok(())
}

#[test]
fn campaign_retry_retains_terminal_history_and_initializes_linked_attempts() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let initial = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-linked-retry");
    ensure_stage_campaign_attempts(temporary.path(), &initial)?;
    for attempt in &initial.cycle.attempts {
        bijux_dna_runtime::record_attempt_ready(
            temporary.path(),
            &attempt.identity,
            "2026-08-05T00:01:00Z",
        )?;
        bijux_dna_runtime::record_attempt_superseded(
            temporary.path(),
            &attempt.identity,
            "2026-08-05T00:02:00Z",
            "scheduler partition authority was rejected before submission",
        )?;
    }
    let prior = initial
        .cycle
        .attempts
        .iter()
        .map(|attempt| load_attempt_lifecycle(temporary.path(), &attempt.identity))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let retry = plan_stage_campaign_retry_attempts(
        &registry,
        &"a".repeat(40),
        &"b".repeat(64),
        prior,
        "attempt-002",
        "2026-08-05T00:03:00Z",
    )?;

    assert_eq!(retry.cycle.attempts.len(), 4);
    let active = retry
        .cycle
        .attempts
        .iter()
        .filter(|attempt| attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned))
        .collect::<Vec<_>>();
    assert_eq!(active.len(), 2);
    assert!(active.iter().all(|attempt| {
        attempt.identity.attempt_id == "attempt-002"
            && attempt.supersedes_attempt_id.as_deref() == Some("attempt-001")
    }));
    let initialized = ensure_stage_campaign_attempts(temporary.path(), &retry)?;
    assert_eq!(initialized.len(), 2);
    for attempt in active {
        assert_eq!(load_attempt_lifecycle(temporary.path(), &attempt.identity)?, *attempt);
    }
    Ok(())
}

#[test]
fn campaign_retry_rejects_nonterminal_predecessors() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let initial = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;

    assert!(plan_stage_campaign_retry_attempts(
        &registry,
        &"a".repeat(40),
        &"b".repeat(64),
        initial.cycle.attempts,
        "attempt-002",
        "2026-08-05T00:03:00Z",
    )
    .is_err());
    Ok(())
}

#[test]
fn campaign_retry_accepts_cancelled_scheduler_predecessors() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let initial = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-cancelled-retry");
    ensure_stage_campaign_attempts(temporary.path(), &initial)?;
    for (index, attempt) in initial.cycle.attempts.iter().enumerate() {
        let job_id = format!("34610{}", index + 1);
        bijux_dna_runtime::record_attempt_ready(
            temporary.path(),
            &attempt.identity,
            "2026-08-05T00:01:00Z",
        )?;
        bijux_dna_runtime::record_attempt_submission(
            temporary.path(),
            &attempt.identity,
            "2026-08-05T00:02:00Z",
            vec![bijux_dna_bench::BenchmarkJobReference {
                scheduler: "slurm".to_string(),
                job_id: job_id.clone(),
            }],
        )?;
        bijux_dna_runtime::record_attempt_accounting(
            temporary.path(),
            &attempt.identity,
            "2026-08-05T00:03:00Z",
            bijux_dna_bench::BenchmarkSchedulerAccounting {
                scheduler: "slurm".to_string(),
                job_id,
                state: bijux_dna_bench::BenchmarkSchedulerTerminalState::Cancelled,
                raw_state: "CANCELLED".to_string(),
                exit_code: 1,
                signal: 0,
                elapsed_seconds: 30,
                total_cpu_seconds: 10,
                max_rss_bytes: Some(1024),
                node_list: "n001".to_string(),
                reason: Some("operator requested scheduler cancellation".to_string()),
            },
        )?;
    }
    let prior = initial
        .cycle
        .attempts
        .iter()
        .map(|attempt| load_attempt_lifecycle(temporary.path(), &attempt.identity))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let retry = plan_stage_campaign_retry_attempts(
        &registry,
        &"a".repeat(40),
        &"b".repeat(64),
        prior,
        "attempt-002",
        "2026-08-05T00:04:00Z",
    )?;

    let active = retry
        .cycle
        .attempts
        .iter()
        .filter(|attempt| attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned))
        .collect::<Vec<_>>();
    assert_eq!(active.len(), 2);
    assert!(active.iter().all(|attempt| {
        attempt.identity.attempt_id == "attempt-002"
            && attempt.supersedes_attempt_id.as_deref() == Some("attempt-001")
    }));
    Ok(())
}

#[test]
fn campaign_retry_recovery_selects_explicit_complete_tool_lineage() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-retry-recovery");
    initialize_retry_recovery_fixture(temporary.path(), &registry)?;

    let lineage = recover_stage_campaign_retry_lineage(
        temporary.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-005",
    )?;

    assert_eq!(lineage.len(), 6);
    assert_eq!(
        lineage
            .iter()
            .map(|record| (record.identity.tool_id.as_str(), record.identity.attempt_id.as_str()))
            .collect::<Vec<_>>(),
        [
            ("seqfu", "attempt-003"),
            ("seqfu", "attempt-004"),
            ("seqfu", "attempt-005"),
            ("seqkit", "attempt-003"),
            ("seqkit", "attempt-004"),
            ("seqkit", "attempt-005"),
        ]
    );
    let retry = plan_stage_campaign_retry_attempts(
        &registry,
        &"a".repeat(40),
        &"b".repeat(64),
        lineage,
        "attempt-006",
        "2026-08-05T01:00:00Z",
    )?;
    assert_eq!(retry.cycle.attempts.len(), 8);
    assert_eq!(
        retry
            .cycle
            .attempts
            .iter()
            .filter(|record| record.current_status() == Some(BenchmarkLifecycleStatus::Planned))
            .count(),
        2
    );
    Ok(())
}

#[test]
fn campaign_retry_recovery_rejects_missing_predecessor_and_cycles() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let missing = bijux_dna_testkit::tempdir_for("benchmark-campaign-missing-predecessor");
    initialize_retry_recovery_fixture(missing.path(), &registry)?;
    rewrite_origin_supersession(
        missing.path(),
        &registry,
        "seqfu",
        "attempt-003",
        Some("attempt-999"),
    )?;
    assert!(recover_stage_campaign_retry_lineage(
        missing.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-005",
    )
    .is_err());

    let cycle = bijux_dna_testkit::tempdir_for("benchmark-campaign-lineage-cycle");
    initialize_retry_recovery_fixture(cycle.path(), &registry)?;
    rewrite_origin_supersession(
        cycle.path(),
        &registry,
        "seqfu",
        "attempt-003",
        Some("attempt-005"),
    )?;
    assert!(recover_stage_campaign_retry_lineage(
        cycle.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-005",
    )
    .is_err());
    Ok(())
}

#[test]
fn campaign_retry_recovery_rejects_nonterminal_and_identity_drift() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let nonterminal = bijux_dna_testkit::tempdir_for("benchmark-campaign-nonterminal-tip");
    let planned = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;
    ensure_stage_campaign_attempts(nonterminal.path(), &planned)?;
    assert!(recover_stage_campaign_retry_lineage(
        nonterminal.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
    )
    .is_err());

    let drift = bijux_dna_testkit::tempdir_for("benchmark-campaign-retry-identity-drift");
    initialize_retry_recovery_fixture(drift.path(), &registry)?;
    rewrite_origin_tool(drift.path(), &registry, "seqfu", "attempt-005", "seqkit")?;
    assert!(recover_stage_campaign_retry_lineage(
        drift.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-005",
    )
    .is_err());

    let run_drift = bijux_dna_testkit::tempdir_for("benchmark-campaign-retry-run-drift");
    initialize_retry_recovery_fixture(run_drift.path(), &registry)?;
    rewrite_origin_run(run_drift.path(), &registry, "seqfu", "attempt-005", "run-2043")?;
    assert!(recover_stage_campaign_retry_lineage(
        run_drift.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-005",
    )
    .is_err());
    Ok(())
}

#[test]
fn campaign_retry_recovery_rejects_detached_execution_evidence() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let temporary = bijux_dna_testkit::tempdir_for("benchmark-campaign-competing-lineage");
    initialize_retry_recovery_fixture(temporary.path(), &registry)?;
    let extra = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-006",
        "2026-08-05T00:20:00Z",
    )?;
    ensure_stage_campaign_attempts(temporary.path(), &extra)?;
    for record in extra.cycle.attempts {
        bijux_dna_runtime::record_attempt_superseded(
            temporary.path(),
            &record.identity,
            "2026-08-05T00:21:00Z",
            "independent terminal lineage retained for ambiguity test",
        )?;
    }

    assert!(recover_stage_campaign_retry_lineage(
        temporary.path(),
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-005",
    )
    .is_err());
    Ok(())
}

fn initialize_retry_recovery_fixture(
    root: &std::path::Path,
    registry: &BenchmarkStageCampaignRegistry,
) -> anyhow::Result<()> {
    for (attempt_id, observed_at) in
        [("attempt-001", "2026-08-05T00:00:00Z"), ("attempt-002", "2026-08-05T00:01:00Z")]
    {
        let detached = plan_stage_campaign_attempts(
            registry,
            "run-2042",
            "current-source-campaign",
            attempt_id,
            observed_at,
        )?;
        ensure_stage_campaign_attempts(root, &detached)?;
    }
    let mut predecessor = None;
    for (attempt_id, planned_at, terminal_at) in [
        ("attempt-003", "2026-08-05T00:02:00Z", "2026-08-05T00:03:00Z"),
        ("attempt-004", "2026-08-05T00:04:00Z", "2026-08-05T00:05:00Z"),
        ("attempt-005", "2026-08-05T00:06:00Z", "2026-08-05T00:07:00Z"),
    ] {
        let mut plan = plan_stage_campaign_attempts(
            registry,
            "run-2042",
            "current-source-campaign",
            attempt_id,
            planned_at,
        )?;
        for record in &mut plan.cycle.attempts {
            record.supersedes_attempt_id = predecessor.map(str::to_string);
        }
        for record in &plan.cycle.attempts {
            bijux_dna_bench::initialize_attempt_lifecycle(root, record)?;
        }
        for record in plan.cycle.attempts {
            bijux_dna_runtime::record_attempt_superseded(
                root,
                &record.identity,
                terminal_at,
                "terminal retry lineage retained for recovery contract test",
            )?;
        }
        predecessor = Some(attempt_id);
    }
    Ok(())
}

fn rewrite_origin_supersession(
    root: &std::path::Path,
    registry: &BenchmarkStageCampaignRegistry,
    tool_id: &str,
    attempt_id: &str,
    supersedes_attempt_id: Option<&str>,
) -> anyhow::Result<()> {
    let identity = retry_fixture_identity(registry, tool_id, attempt_id)?;
    let path = benchmark_attempt_directory(root, &identity)?.join("attempt.json");
    let mut record: bijux_dna_bench::BenchmarkAttemptRecordV2 =
        serde_json::from_slice(&std::fs::read(&path)?)?;
    record.supersedes_attempt_id = supersedes_attempt_id.map(str::to_string);
    std::fs::write(path, serde_json::to_vec_pretty(&record)?)?;
    Ok(())
}

fn rewrite_origin_tool(
    root: &std::path::Path,
    registry: &BenchmarkStageCampaignRegistry,
    tool_id: &str,
    attempt_id: &str,
    replacement: &str,
) -> anyhow::Result<()> {
    let identity = retry_fixture_identity(registry, tool_id, attempt_id)?;
    let path = benchmark_attempt_directory(root, &identity)?.join("attempt.json");
    let mut record: bijux_dna_bench::BenchmarkAttemptRecordV2 =
        serde_json::from_slice(&std::fs::read(&path)?)?;
    record.identity.tool_id = replacement.to_string();
    std::fs::write(path, serde_json::to_vec_pretty(&record)?)?;
    Ok(())
}

fn rewrite_origin_run(
    root: &std::path::Path,
    registry: &BenchmarkStageCampaignRegistry,
    tool_id: &str,
    attempt_id: &str,
    replacement: &str,
) -> anyhow::Result<()> {
    let identity = retry_fixture_identity(registry, tool_id, attempt_id)?;
    let path = benchmark_attempt_directory(root, &identity)?.join("attempt.json");
    let mut record: bijux_dna_bench::BenchmarkAttemptRecordV2 =
        serde_json::from_slice(&std::fs::read(&path)?)?;
    record.identity.run.run_id = replacement.to_string();
    std::fs::write(path, serde_json::to_vec_pretty(&record)?)?;
    Ok(())
}

fn retry_fixture_identity(
    registry: &BenchmarkStageCampaignRegistry,
    tool_id: &str,
    attempt_id: &str,
) -> anyhow::Result<bijux_dna_bench::BenchmarkAttemptIdentity> {
    let plan = plan_stage_campaign_attempts(
        registry,
        "run-2042",
        "current-source-campaign",
        attempt_id,
        "2026-08-05T00:00:00Z",
    )?;
    plan.cycle
        .attempts
        .into_iter()
        .find(|record| record.identity.tool_id == tool_id)
        .map(|record| record.identity)
        .ok_or_else(|| anyhow::anyhow!("retry fixture tool is absent"))
}

#[test]
fn checked_in_campaign_plan_schema_matches_model_authority() -> anyhow::Result<()> {
    let path = bijux_dna_bench::bench_contracts_dir().join("schemas/campaign-plan.schema.json");
    let checked_in: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let generated = serde_json::to_value(bijux_dna_bench::benchmark_campaign_plan_schema())?;

    assert_eq!(checked_in, generated);
    Ok(())
}

#[test]
fn checked_in_scheduler_observation_schema_matches_model_authority() -> anyhow::Result<()> {
    let path =
        bijux_dna_bench::bench_contracts_dir().join("schemas/scheduler-observation.schema.json");
    let checked_in: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let generated =
        serde_json::to_value(bijux_dna_bench::benchmark_scheduler_observation_schema())?;

    assert_eq!(checked_in, generated);
    Ok(())
}

#[test]
fn campaign_plan_source_verification_rejects_commit_and_registry_drift() -> anyhow::Result<()> {
    let (registry, _, _) = load_checked_in_benchmark_planning_contracts()?;
    let plan = plan_stage_campaign_attempts(
        &registry,
        "run-2042",
        "current-source-campaign",
        "attempt-001",
        "2026-08-05T00:00:00Z",
    )?;

    verify_campaign_plan_source(&plan, &"a".repeat(40), &"b".repeat(64))?;
    assert!(verify_campaign_plan_source(&plan, &"c".repeat(40), &"b".repeat(64)).is_err());
    assert!(verify_campaign_plan_source(&plan, &"a".repeat(40), &"d".repeat(64)).is_err());
    Ok(())
}
