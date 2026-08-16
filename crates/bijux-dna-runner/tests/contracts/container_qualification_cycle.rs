use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_runner::api::{
    advance_container_qualification_cycle, plan_container_qualification_cycle,
    ContainerQualificationCycleAction, ContainerQualificationCyclePreparation,
    ContainerQualificationCycleRequest, ContainerQualificationCycleState,
    ContainerQualificationSourceMaterialization, SlurmClientPaths,
    CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1,
    CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V2,
    CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2,
};

use crate::support::repo_root;

#[test]
fn cycle_plan_and_closed_frontier_advance_are_idempotent() -> Result<()> {
    let repository_root = repo_root()?;
    let empty_submissions = bijux_dna_infra::temp_dir("container-cycle-empty-submissions")?;
    let evidence: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository_root.join("containers/runs/run-1002-adapterremoval/evidence.lock.json"),
    )?)?;
    let adapterremoval_sha256 = evidence["image"]["sha256"]
        .as_str()
        .context("adapterremoval evidence omits image identity")?;
    let inventory_root = bijux_dna_infra::temp_dir("container-cycle-image-inventory")?;
    let inventory_path = source_current_inventory(
        &repository_root,
        inventory_root.path(),
        &[("adapterremoval", adapterremoval_sha256)],
    )?;
    let request = ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        inventory_path,
        publication_root: repository_root.join("containers/runs"),
        submission_root: empty_submissions.path().to_path_buf(),
        external_artifact_admission_root: None,
        preparation: None,
        observed_at_utc: "2026-08-09T12:00:00Z".to_string(),
    };

    let first = plan_container_qualification_cycle(&request)?;
    let second = plan_container_qualification_cycle(&request)?;
    let advanced = advance_container_qualification_cycle(&SlurmClientPaths::default(), &request)?;
    assert_eq!(first, second);
    assert_eq!(first, advanced);
    assert_eq!(first.schema_version, CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2);
    assert_eq!(first.summary.unit_count, 89);
    assert_eq!(first.summary.published_count, 0);
    assert_eq!(first.summary.blocked_count, 89);
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
            == 1
    );
    let adapterremoval = first
        .units
        .iter()
        .find(|unit| unit.tool_id == "adapterremoval")
        .context("qualification cycle omits adapterremoval")?;
    assert_eq!(adapterremoval.next_action, ContainerQualificationCycleAction::Prepare);
    assert!(adapterremoval
        .detail
        .as_deref()
        .is_some_and(|detail| detail.contains("differs from current repository commit")));
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
fn cycle_plan_blocks_invalid_preparation_sources_before_submission() -> Result<()> {
    let repository_root = repo_root()?;
    let submissions = bijux_dna_infra::temp_dir("container-cycle-preparable-submissions")?;
    let sources = bijux_dna_infra::temp_dir("container-cycle-preparable-sources")?;
    let authorities = bijux_dna_infra::temp_dir("container-cycle-preparable-authorities")?;
    for name in ["governance.json", "credential.json", "bijux-dna-dev"] {
        std::fs::write(authorities.path().join(name), b"retained authority\n")?;
    }
    std::fs::write(sources.path().join("adapterremoval.json"), b"{}\n")?;
    let evidence: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository_root.join("containers/runs/run-1002-adapterremoval/evidence.lock.json"),
    )?)?;
    let image_sha256 = evidence["image"]["sha256"]
        .as_str()
        .context("adapterremoval evidence omits image identity")?;
    let inventory_root = bijux_dna_infra::temp_dir("container-cycle-preparable-inventory")?;
    let inventory_path = source_current_inventory(
        &repository_root,
        inventory_root.path(),
        &[("adapterremoval", image_sha256)],
    )?;
    let status = plan_container_qualification_cycle(&ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        inventory_path,
        publication_root: repository_root.join("containers/runs"),
        submission_root: submissions.path().to_path_buf(),
        external_artifact_admission_root: None,
        preparation: Some(ContainerQualificationCyclePreparation {
            schema_version: CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1.to_string(),
            sources_root: sources.path().to_path_buf(),
            governance_path: authorities.path().join("governance.json"),
            credential_reference_path: authorities.path().join("credential.json"),
            executor_program_path: authorities.path().join("bijux-dna-dev"),
            hostname: "lunarc".to_string(),
            source_materialization: None,
        }),
        observed_at_utc: "2026-08-13T22:00:00Z".to_string(),
    })?;

    let unit = status
        .units
        .iter()
        .find(|unit| unit.tool_id == "adapterremoval")
        .context("qualification cycle omits adapterremoval")?;
    assert_eq!(unit.state, ContainerQualificationCycleState::Blocked);
    assert_eq!(unit.next_action, ContainerQualificationCycleAction::None);
    assert!(unit
        .detail
        .as_deref()
        .is_some_and(|detail| detail.contains("preparation source is invalid")));
    assert_eq!(status.summary.ready_count, 0);
    Ok(())
}

#[test]
fn cycle_plan_exposes_each_declared_source_as_an_independent_materialization_unit() -> Result<()> {
    let repository_root = repo_root()?;
    let submissions = bijux_dna_infra::temp_dir("container-cycle-materialized-submissions")?;
    let sources = bijux_dna_infra::temp_dir("container-cycle-materialized-sources")?;
    let selections = bijux_dna_infra::temp_dir("container-cycle-materialized-selections")?;
    let authorities = bijux_dna_infra::temp_dir("container-cycle-materialized-authorities")?;
    for name in ["governance.json", "credential.json", "bijux-dna-dev", "data-index.json"] {
        std::fs::write(authorities.path().join(name), b"retained authority\n")?;
    }
    let selection = bijux_dna_containers_qa::api::QualificationSourceSelectionV1 {
        schema_version: bijux_dna_containers_qa::api::QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1
            .to_string(),
        tool_id: "adapterremoval".to_string(),
        campaign: "canonical-stage-qualification".to_string(),
        corpus: "governed-fastq".to_string(),
        corpus_role: "fastq".to_string(),
        sample: "sample-1".to_string(),
        inputs: vec![bijux_dna_containers_qa::api::QualificationSourceInputSelectionV1 {
            binding_input: "reads_r1".to_string(),
            id: "governed_fastq_r1".to_string(),
            kind: bijux_dna_containers_qa::api::QualificationSpecificationInputKindV1::Data,
            catalog_id: "governed-fastq".to_string(),
            study_uri: "study://data/corpora/governed-fastq/reads.fastq.gz".to_string(),
            mount_path: PathBuf::from("/input/reads-r1.fastq.gz"),
        }],
        parameters: BTreeMap::from([("threads".to_string(), "4".to_string())]),
        negative_arguments: vec!["--definitely-invalid".to_string()],
    };
    bijux_dna_infra::atomic_create_json(
        &selections.path().join("adapterremoval.json"),
        &selection,
    )?;
    let evidence: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository_root.join("containers/runs/run-1002-adapterremoval/evidence.lock.json"),
    )?)?;
    let image_sha256 = evidence["image"]["sha256"]
        .as_str()
        .context("adapterremoval evidence omits image identity")?;
    let inventory_root = bijux_dna_infra::temp_dir("container-cycle-materialized-inventory")?;
    let inventory_path = source_current_inventory(
        &repository_root,
        inventory_root.path(),
        &[("adapterremoval", image_sha256)],
    )?;
    let status = plan_container_qualification_cycle(&ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        inventory_path,
        publication_root: repository_root.join("containers/runs"),
        submission_root: submissions.path().to_path_buf(),
        external_artifact_admission_root: None,
        preparation: Some(ContainerQualificationCyclePreparation {
            schema_version: CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V2.to_string(),
            sources_root: sources.path().to_path_buf(),
            governance_path: authorities.path().join("governance.json"),
            credential_reference_path: authorities.path().join("credential.json"),
            executor_program_path: authorities.path().join("bijux-dna-dev"),
            hostname: "lunarc".to_string(),
            source_materialization: Some(ContainerQualificationSourceMaterialization {
                selections_root: selections.path().to_path_buf(),
                data_index_path: authorities.path().join("data-index.json"),
                operator: "bijux-dna-dev".to_string(),
                walltime: "00:30:00".to_string(),
            }),
        }),
        observed_at_utc: "2026-08-13T22:00:00Z".to_string(),
    })?;
    let unit = status
        .units
        .iter()
        .find(|unit| unit.tool_id == "adapterremoval")
        .context("qualification cycle omits adapterremoval")?;
    assert_eq!(unit.state, ContainerQualificationCycleState::Ready);
    assert_eq!(unit.next_action, ContainerQualificationCycleAction::Materialize);
    Ok(())
}

#[test]
fn cycle_plan_verifies_governed_external_access_blockers() -> Result<()> {
    let repository_root = repo_root()?;
    let submissions = bijux_dna_infra::temp_dir("container-cycle-blocked-submissions")?;
    let admissions = bijux_dna_infra::temp_dir("container-cycle-external-admissions")?;
    let image_root = bijux_dna_infra::temp_dir("container-cycle-active-images")?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)?;
    let inventory_root = bijux_dna_infra::temp_dir("container-cycle-image-inventory")?;
    let inventory_path = source_current_inventory(&repository_root, inventory_root.path(), &[])?;
    bijux_dna_containers::api::admit_external_artifact(
        &bijux_dna_containers::api::ExternalArtifactAdmissionRequest {
            repository_root: &repository_root,
            distributable_image_root: image_root.path(),
            repository_commit: &repository_commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-09T12:00:00Z",
            availability:
                bijux_dna_containers::api::ExternalArtifactAdmissionStatus::BlockedExternalAccess,
            artifact: None,
            blocker_reason: Some("institutional artifact access has not been supplied"),
        },
        &admissions.path().join("angsd.json"),
    )?;
    let status = plan_container_qualification_cycle(&ContainerQualificationCycleRequest {
        repository_root: repository_root.clone(),
        registry_path: repository_root.join("containers/contracts/qualification-runs.toml"),
        inventory_path,
        publication_root: repository_root.join("containers/runs"),
        submission_root: submissions.path().to_path_buf(),
        external_artifact_admission_root: Some(admissions.path().to_path_buf()),
        preparation: None,
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
        Some("external artifact access is blocked: institutional artifact access has not been supplied")
    );
    Ok(())
}

fn source_current_inventory(
    repository_root: &Path,
    root: &Path,
    current: &[(&str, &str)],
) -> Result<PathBuf> {
    let path = root.join("inventory.json");
    let catalog = bijux_dna_containers::api::load_container_catalog(repository_root)?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(repository_root)?;
    let selected = current.iter().copied().collect::<BTreeMap<_, _>>();
    let mut states = BTreeMap::new();
    let tools = catalog
        .tools
        .iter()
        .map(|tool| {
            let image_sha256 = selected.get(tool.id.as_str()).copied();
            let state = if image_sha256.is_some() {
                bijux_dna_containers::api::ContainerImageState::Current
            } else if tool.artifact_delivery
                == bijux_dna_containers::api::ContainerArtifactDelivery::RedistributableImage
            {
                bijux_dna_containers::api::ContainerImageState::Missing
            } else {
                bijux_dna_containers::api::ContainerImageState::NotApplicable
            };
            let state_name = match state {
                bijux_dna_containers::api::ContainerImageState::Current => "current",
                bijux_dna_containers::api::ContainerImageState::Missing => "missing",
                bijux_dna_containers::api::ContainerImageState::NotApplicable => "not_applicable",
                _ => unreachable!(),
            };
            *states.entry(state_name.to_string()).or_insert(0_u64) += 1;
            bijux_dna_containers::api::ContainerImageInventoryTool {
                tool: tool.id.clone(),
                version: tool.version.clone(),
                product_support: tool.product_support,
                artifact_delivery: tool.artifact_delivery,
                redistribution_permitted: tool.redistribution_permitted,
                apptainer_supported: tool.apptainer_supported,
                state,
                selected_image_sha256: image_sha256.map(str::to_string),
                candidates: image_sha256
                    .map(|sha256| {
                        vec![bijux_dna_containers::api::ContainerImageCandidate {
                            admitted: true,
                            current_recipe: true,
                            disposition:
                                bijux_dna_containers::api::ImageCandidateDisposition::Canonical,
                            path: format!("study://containers/current-images/{}.sif", tool.id),
                            sha256: sha256.to_string(),
                            size_bytes: 1,
                            real_run_references: 0,
                            run_references: Vec::new(),
                            build_evidence: Vec::new(),
                        }]
                    })
                    .unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();
    let publication = bijux_dna_containers::api::ContainerImageInventoryPublication {
        schema_version: bijux_dna_containers::api::CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3
            .to_string(),
        catalog_sha256: bijux_dna_infra::hash_file_sha256(
            &repository_root.join("containers/contracts/catalog.toml"),
        )?,
        repository_commit,
        publication: bijux_dna_containers::api::ImageInventoryPublicationIdentity {
            raw_schema_version: "bijux.container.image_inventory_scan.v2".to_string(),
            raw_sha256: "a".repeat(64),
        },
        summary: bijux_dna_containers::api::ContainerImageInventorySummary {
            tool_count: u64::try_from(tools.len())?,
            states,
            historical_dependency_count: 0,
            unknown_image_count: 0,
            staged_image_count: 0,
            unreferenced_obsolete_count: 0,
        },
        tools,
        historical_images: Vec::new(),
        unknown_images: Vec::new(),
        staged_images: Vec::new(),
    };
    bijux_dna_infra::atomic_create_json(&path, &publication)?;
    Ok(path)
}
