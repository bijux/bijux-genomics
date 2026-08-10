use anyhow::{Context, Result};
use bijux_dna_runner::api::{
    plan_container_qualification_cycle, ContainerQualificationCycleAction,
    ContainerQualificationCycleRequest, ContainerQualificationCycleState,
    CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2,
};

use crate::support::repo_root;

#[test]
fn cycle_plan_is_idempotent_and_exposes_every_current_protocol_action() -> Result<()> {
    let repository_root = repo_root()?;
    let empty_submissions = bijux_dna_infra::temp_dir("container-cycle-empty-submissions")?;
    let legacy = empty_submissions.path().join("run-1002-adapterremoval");
    std::fs::create_dir_all(&legacy)?;
    std::fs::write(
        legacy.join("qualification-spec.json"),
        br#"{
  "schema_version": "bijux.runner.container_qualification_slurm_spec.v1",
  "run_id": "run-1002",
  "tool_id": "adapterremoval",
  "executor_script_path": "containers/contracts/container_validation.py"
}
"#,
    )?;
    let request = ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        publication_root: repository_root.join("containers/runs"),
        submission_root: empty_submissions.path().to_path_buf(),
        external_artifact_admission_root: None,
        observed_at_utc: "2026-08-09T12:00:00Z".to_string(),
    };

    let first = plan_container_qualification_cycle(&request)?;
    let second = plan_container_qualification_cycle(&request)?;
    assert_eq!(first, second);
    assert_eq!(first.schema_version, CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2);
    assert_eq!(first.summary.unit_count, 89);
    assert_eq!(first.summary.published_count, 68);
    assert_eq!(first.summary.blocked_count, 21);
    assert_eq!(first.summary.ready_count, 0);
    assert_eq!(first.summary.active_count, 0);
    assert_eq!(first.earliest_observation_at_utc, None);
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
    assert_eq!(
        first
            .units
            .iter()
            .filter(|unit| {
                unit.state == ContainerQualificationCycleState::Blocked
                    && unit.next_action == ContainerQualificationCycleAction::None
                    && unit
                        .detail
                        .as_deref()
                        .is_some_and(|detail| detail.contains("external artifact"))
            })
            .count(),
        4
    );
    Ok(())
}

#[test]
fn cycle_plan_verifies_governed_external_access_blockers() -> Result<()> {
    let repository_root = repo_root()?;
    let submissions = bijux_dna_infra::temp_dir("container-cycle-blocked-submissions")?;
    let admissions = bijux_dna_infra::temp_dir("container-cycle-external-admissions")?;
    let image_root = bijux_dna_infra::temp_dir("container-cycle-active-images")?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)?;
    bijux_dna_containers::api::admit_external_artifact(
        &bijux_dna_containers::api::ExternalArtifactAdmissionRequest {
            repository_root: &repository_root,
            distributable_image_root: image_root.path(),
            repository_commit: &repository_commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-09T12:00:00Z",
            artifact: None,
            blocker_reason: Some("institutional artifact access has not been supplied"),
        },
        &admissions.path().join("angsd.json"),
    )?;
    let status = plan_container_qualification_cycle(&ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        publication_root: repository_root.join("containers/runs"),
        submission_root: submissions.path().to_path_buf(),
        external_artifact_admission_root: Some(admissions.path().to_path_buf()),
        observed_at_utc: "2026-08-09T12:00:00Z".to_string(),
    })?;

    let angsd = status
        .units
        .iter()
        .find(|unit| unit.tool_id == "angsd")
        .context("qualification cycle omits ANGSD")?;
    assert_eq!(angsd.run_id, "run-1094");
    assert_eq!(angsd.state, ContainerQualificationCycleState::Blocked);
    assert_eq!(angsd.next_action, ContainerQualificationCycleAction::None);
    assert_eq!(
        angsd.detail.as_deref(),
        Some(
            "external artifact access is blocked: institutional artifact access has not been supplied"
        )
    );
    Ok(())
}
