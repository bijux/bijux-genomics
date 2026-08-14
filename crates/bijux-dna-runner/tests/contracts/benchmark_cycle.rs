use std::path::PathBuf;

use bijux_dna_bench_model::{
    BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2, BenchmarkCampaignPlan,
    BenchmarkCycleStatusV2, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
};
use bijux_dna_runner::api::{
    collect_benchmark_campaign_cycle, submit_benchmark_campaign_frontier,
    BenchmarkCampaignSubmissionState, SlurmClientPaths,
};

fn planned_campaign() -> BenchmarkCampaignPlan {
    let identity = BenchmarkAttemptIdentity {
        campaign_id: "campaign-run-2002".to_string(),
        run: BenchmarkRunIdentity {
            run_id: "run-2002".to_string(),
            domain: "bam".to_string(),
            stage_id: "bam.coverage".to_string(),
        },
        case_id: None,
        attempt_id: "attempt-0001".to_string(),
        tool_id: "fastp".to_string(),
    };
    BenchmarkCampaignPlan::new(
        "a".repeat(40),
        "b".repeat(64),
        BenchmarkCycleStatusV2::new(
            identity.campaign_id.clone(),
            "2026-08-10T00:00:00Z".to_string(),
            vec![BenchmarkAttemptRecordV2::planned(identity, "2026-08-10T00:00:00Z".to_string())],
        ),
    )
}

#[test]
fn planned_attempts_return_without_scheduler_or_file_polling() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("benchmark-campaign-collection")?;
    let plan = planned_campaign();
    bijux_dna_runtime::ensure_attempt_lifecycle(temporary.path(), &plan.cycle.attempts[0])?;
    let unavailable = PathBuf::from("/scheduler-command-must-not-run");
    let paths = SlurmClientPaths {
        sbatch: unavailable.clone(),
        squeue: unavailable.clone(),
        sacct: unavailable.clone(),
        scancel: unavailable,
    };

    let collection =
        collect_benchmark_campaign_cycle(&paths, temporary.path(), &plan, "2026-08-10T00:01:00Z")?;
    assert_eq!(collection.units.len(), 1);
    assert_eq!(collection.units[0].prior_status, BenchmarkLifecycleStatus::Planned);
    assert_eq!(collection.units[0].current_status, BenchmarkLifecycleStatus::Planned);
    assert!(collection.units[0].scheduler_observation.is_none());
    Ok(())
}

#[test]
fn submission_frontier_skips_planned_attempts_without_scheduler_effects() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("benchmark-campaign-submission")?;
    let plan = planned_campaign();
    bijux_dna_runtime::ensure_attempt_lifecycle(temporary.path(), &plan.cycle.attempts[0])?;
    let unavailable = PathBuf::from("/scheduler-command-must-not-run");
    let paths = SlurmClientPaths {
        sbatch: unavailable.clone(),
        squeue: unavailable.clone(),
        sacct: unavailable.clone(),
        scancel: unavailable,
    };

    let submission = submit_benchmark_campaign_frontier(
        &paths,
        temporary.path(),
        &plan,
        "2026-08-10T00:01:00Z",
        1,
    )?;
    assert_eq!(submission.units.len(), 1);
    assert_eq!(submission.units[0].state, BenchmarkCampaignSubmissionState::NotReady);
    assert_eq!(submission.units[0].current_status, BenchmarkLifecycleStatus::Planned);
    Ok(())
}

#[test]
fn submission_frontier_rejects_an_unbounded_zero_limit() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("benchmark-campaign-zero-submission")?;
    assert!(submit_benchmark_campaign_frontier(
        &SlurmClientPaths::default(),
        temporary.path(),
        &planned_campaign(),
        "2026-08-10T00:01:00Z",
        0,
    )
    .is_err());
    Ok(())
}
