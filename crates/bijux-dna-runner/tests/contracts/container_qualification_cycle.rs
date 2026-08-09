use anyhow::Result;
use bijux_dna_runner::api::{
    plan_container_qualification_cycle, ContainerQualificationCycleAction,
    ContainerQualificationCycleRequest, ContainerQualificationCycleState,
    CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1,
};

use crate::support::repo_root;

#[test]
fn cycle_plan_is_idempotent_and_exposes_every_current_protocol_action() -> Result<()> {
    let repository_root = repo_root()?;
    let empty_submissions = bijux_dna_infra::temp_dir("container-cycle-empty-submissions")?;
    let request = ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        publication_root: repository_root.join("containers/runs"),
        submission_root: empty_submissions.path().to_path_buf(),
        observed_at_utc: "2026-08-09T12:00:00Z".to_string(),
    };

    let first = plan_container_qualification_cycle(&request)?;
    let second = plan_container_qualification_cycle(&request)?;
    assert_eq!(first, second);
    assert_eq!(first.schema_version, CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1);
    assert_eq!(first.summary.unit_count, 85);
    assert_eq!(first.summary.published_count, 68);
    assert_eq!(first.summary.blocked_count, 17);
    assert_eq!(first.summary.ready_count, 0);
    assert_eq!(first.summary.active_count, 0);
    assert_eq!(first.earliest_poll_at_utc, None);
    assert!(
        first
            .units
            .iter()
            .filter(|unit| {
                unit.state == ContainerQualificationCycleState::Blocked
                    && unit.next_action == ContainerQualificationCycleAction::Prepare
            })
            .count()
            == 17
    );
    Ok(())
}
