use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::{
    apply_image_activation_plan, create_image_activation_plan, load_image_activation_plan,
    rollback_image_activation_plan, verify_image_activation_plan, ContainerArtifactDelivery,
    ContainerImageCandidate, ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerProductSupport,
    ImageActivationPlanRequest, ImageActivationPlanState, ImageCandidateDisposition,
    ImageInventoryPublicationIdentity, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3,
};

#[test]
fn activation_is_no_clobber_resumable_verified_and_reversible() -> Result<()> {
    let fixture = Fixture::new()?;
    let plan = fixture.create_plan()?;
    assert_eq!(plan.file_count, 2);
    assert!(fixture.create_plan().is_err());

    let applied = apply_image_activation_plan(
        &fixture.plan,
        &fixture.staging,
        &fixture.active,
        "2026-08-05T10:00:00Z",
    )?;
    assert_eq!(applied.state, ImageActivationPlanState::Applied);
    verify_image_activation_plan(&fixture.plan, &fixture.staging, &fixture.active)?;
    assert_eq!(std::fs::read(fixture.active.join("alpha.sif"))?, b"alpha");
    assert_eq!(std::fs::read(fixture.staging.join("alpha.sif"))?, b"alpha");

    let rolled_back = rollback_image_activation_plan(
        &fixture.plan,
        &fixture.staging,
        &fixture.active,
        "2026-08-05T10:01:00Z",
    )?;
    assert_eq!(rolled_back.state, ImageActivationPlanState::RolledBack);
    assert!(!fixture.active.join("alpha.sif").exists());
    assert!(fixture.staging.join("alpha.sif").exists());
    Ok(())
}

#[test]
fn activation_rejects_conflicts_and_source_drift() -> Result<()> {
    let fixture = Fixture::new()?;
    std::fs::write(fixture.active.join("alpha.sif"), b"conflict")?;
    assert!(fixture.create_plan().is_err());
    std::fs::remove_file(fixture.active.join("alpha.sif"))?;

    fixture.create_plan()?;
    std::fs::write(fixture.staging.join("alpha.sif"), b"drifted")?;
    assert!(apply_image_activation_plan(
        &fixture.plan,
        &fixture.staging,
        &fixture.active,
        "2026-08-05T10:00:00Z",
    )
    .is_err());
    assert_eq!(
        load_image_activation_plan(&fixture.plan)?.state,
        ImageActivationPlanState::Applying
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn interrupted_application_accepts_only_the_planned_hard_link() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.create_plan()?;
    std::fs::hard_link(fixture.staging.join("alpha.sif"), fixture.active.join("alpha.sif"))?;
    let applied = apply_image_activation_plan(
        &fixture.plan,
        &fixture.staging,
        &fixture.active,
        "2026-08-05T10:00:00Z",
    )?;
    assert!(applied.publications.iter().all(|row| row.applied));
    Ok(())
}

struct Fixture {
    _temporary: tempfile::TempDir,
    study: PathBuf,
    staging: PathBuf,
    active: PathBuf,
    inventory: PathBuf,
    plan: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self> {
        let temporary = tempfile::tempdir()?;
        let study = temporary.path().join("STUDY");
        let staging = study.join("containers/staging/commit");
        let active = study.join("containers/apptainer");
        let evidence = study.join("containers/evidence/activation/commit");
        std::fs::create_dir_all(&staging)?;
        std::fs::create_dir_all(&active)?;
        std::fs::create_dir_all(&evidence)?;
        std::fs::write(staging.join("alpha.sif"), b"alpha")?;
        std::fs::write(staging.join("beta.sif"), b"beta")?;
        let inventory = evidence.join("image-inventory.json");
        write_inventory(&inventory, &study, &staging)?;
        Ok(Self {
            _temporary: temporary,
            study,
            staging,
            active,
            inventory,
            plan: evidence.join("image-activation-plan.json"),
        })
    }

    fn create_plan(&self) -> Result<bijux_dna_containers::api::ImageActivationPlan> {
        create_image_activation_plan(
            &ImageActivationPlanRequest {
                inventory_path: &self.inventory,
                study_root: &self.study,
                source_root: &self.staging,
                active_root: &self.active,
                created_at_utc: "2026-08-05T09:59:00Z",
            },
            &self.plan,
        )
    }
}

fn write_inventory(path: &Path, study: &Path, staging: &Path) -> Result<()> {
    let tools = ["alpha", "beta"]
        .into_iter()
        .map(|tool| {
            let image = staging.join(format!("{tool}.sif"));
            let sha256 = bijux_dna_infra::hash_file_sha256(&image)?;
            Ok(ContainerImageInventoryTool {
                tool: tool.to_string(),
                version: "1.0.0".to_string(),
                product_support: ContainerProductSupport::Supported,
                artifact_delivery: ContainerArtifactDelivery::RedistributableImage,
                redistribution_permitted: true,
                apptainer_supported: true,
                state: ContainerImageState::Current,
                selected_image_sha256: Some(sha256.clone()),
                candidates: vec![ContainerImageCandidate {
                    admitted: true,
                    current_recipe: true,
                    disposition: ImageCandidateDisposition::Canonical,
                    path: locator(study, &image),
                    sha256,
                    size_bytes: std::fs::metadata(image)?.len(),
                    real_run_references: 0,
                    run_references: Vec::new(),
                    build_evidence: Vec::new(),
                }],
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let publication = ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3.to_string(),
        catalog_sha256: "b".repeat(64),
        repository_commit: "a".repeat(40),
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: "bijux.container.image_inventory_scan.v1".to_string(),
            raw_sha256: "c".repeat(64),
        },
        summary: ContainerImageInventorySummary {
            tool_count: 2,
            states: BTreeMap::from([("current".to_string(), 2)]),
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
    bijux_dna_infra::atomic_create_json(path, &publication)?;
    Ok(())
}

fn locator(study: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(study).map_or(path, |relative| relative);
    format!("study://{}", relative.display())
}
