use bijux_dna_bench::{
    benchmark_attempt_directory, ensure_stage_campaign_attempts, load_attempt_lifecycle,
    load_checked_in_benchmark_planning_contracts, plan_stage_campaign_attempts,
    BenchmarkLifecycleStatus,
};

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

    assert_eq!(cycle.attempts.len(), 2);
    assert_eq!(
        cycle.attempts.iter().map(|attempt| attempt.identity.tool_id.as_str()).collect::<Vec<_>>(),
        ["seqfu", "seqkit"]
    );
    assert!(cycle.attempts.iter().all(|attempt| {
        attempt.identity.attempt_id == "attempt-001"
            && attempt.identity.run.run_id == "run-2042"
            && attempt.identity.run.domain == "fastq"
            && attempt.identity.run.stage_id == "fastq.profile_read_lengths"
            && attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned)
    }));
    for attempt in &cycle.attempts {
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
    for attempt in &cycle.attempts {
        assert_eq!(load_attempt_lifecycle(temporary.path(), &attempt.identity)?, *attempt);
    }
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
    bijux_dna_bench::initialize_attempt_lifecycle(temporary.path(), &conflicting.attempts[1])?;

    assert!(ensure_stage_campaign_attempts(temporary.path(), &cycle).is_err());
    let missing = benchmark_attempt_directory(temporary.path(), &cycle.attempts[0].identity)?;
    assert!(!missing.exists());
    Ok(())
}
