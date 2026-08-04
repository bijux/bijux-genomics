use bijux_dna_bench::{
    benchmark_attempt_directory, load_checked_in_benchmark_planning_contracts,
    plan_stage_campaign_attempts, BenchmarkLifecycleStatus,
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
