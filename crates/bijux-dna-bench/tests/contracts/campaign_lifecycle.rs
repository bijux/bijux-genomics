use bijux_dna_bench::{
    benchmark_attempt_directory, ensure_stage_campaign_attempts, load_attempt_lifecycle,
    load_checked_in_benchmark_planning_contracts,
    plan_stage_campaign_attempts as derive_campaign_plan, plan_stage_campaign_retry_attempts,
    verify_campaign_plan_source, BenchmarkCampaignPlan, BenchmarkLifecycleStatus,
    BenchmarkStageCampaignRegistry,
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
fn checked_in_campaign_plan_schema_matches_model_authority() -> anyhow::Result<()> {
    let path = bijux_dna_bench::bench_contracts_dir().join("schemas/campaign-plan.schema.json");
    let checked_in: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let generated = serde_json::to_value(bijux_dna_bench::benchmark_campaign_plan_schema())?;

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
