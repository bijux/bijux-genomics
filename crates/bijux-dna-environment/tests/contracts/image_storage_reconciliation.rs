use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_environment::api::{
    apply_image_storage_plan, create_image_storage_plan, derive_image_storage_report,
    load_image_storage_plan, publish_image_storage_report, render_image_storage_report_markdown,
    rollback_image_storage_plan, verify_image_storage_plan, ContainerDetachedImage,
    ContainerImageCandidate, ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerStagedImage,
    ContainerSupportDisposition, DetachedImageDisposition, ImageCandidateDisposition,
    ImageInventoryPublicationIdentity, ImageStoragePlanRequest, ImageStoragePlanState,
    ImageStorageVerdict, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2,
};

#[test]
fn quarantine_plan_is_resumable_verified_and_reversible() -> Result<()> {
    let fixture = Fixture::new()?;
    let plan = fixture.create_plan()?;
    assert_eq!(plan.file_count, 5);

    let applied = apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z")?;
    assert_eq!(applied.state, ImageStoragePlanState::Applied);
    verify_image_storage_plan(&fixture.plan, &fixture.images)?;
    assert!(!fixture.obsolete.exists());
    assert!(!fixture.interrupted.exists());
    assert!(!fixture.detached.exists());
    assert!(!fixture.historical.exists());
    assert!(!fixture.detached_historical.exists());

    let rolled_back =
        rollback_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:01:00Z")?;
    assert_eq!(rolled_back.state, ImageStoragePlanState::RolledBack);
    assert_eq!(std::fs::read(&fixture.obsolete)?, b"obsolete");
    assert_eq!(std::fs::read(&fixture.interrupted)?, b"interrupted");
    assert_eq!(std::fs::read(&fixture.detached)?, b"detached");
    assert_eq!(std::fs::read(&fixture.historical)?, b"historical");
    assert_eq!(std::fs::read(&fixture.detached_historical)?, b"detached-historical");
    Ok(())
}

#[test]
fn report_requires_canonical_post_storage_inventory() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.create_plan()?;
    apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z")?;
    write_inventory(&fixture.after_inventory, &fixture, false)?;

    let report = derive_image_storage_report(&fixture.plan, &fixture.after_inventory)?;
    let markdown = render_image_storage_report_markdown(&report);

    assert_eq!(report.verdict, ImageStorageVerdict::Passed);
    assert_eq!(report.moved_file_count, 5);
    assert_eq!(report.historical_dependencies_preserved, 2);
    assert!(report.checks.values().all(|value| *value));
    assert!(markdown.contains("No SIF was deleted"));
    assert!(markdown.contains("containers/apptainer/.quarantine"));
    Ok(())
}

#[test]
fn report_publication_resumes_an_exact_partial_pair_and_rejects_drift() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.create_plan()?;
    apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z")?;
    write_inventory(&fixture.after_inventory, &fixture, false)?;
    let json_output = fixture.plan.with_file_name("image-storage-report.json");
    let markdown_output = fixture.plan.with_file_name("image-storage-report.md");
    let report = derive_image_storage_report(&fixture.plan, &fixture.after_inventory)?;
    bijux_dna_infra::atomic_create_bytes(&json_output, &serde_json::to_vec_pretty(&report)?)?;

    publish_image_storage_report(
        &fixture.plan,
        &fixture.after_inventory,
        &json_output,
        &markdown_output,
    )?;
    publish_image_storage_report(
        &fixture.plan,
        &fixture.after_inventory,
        &json_output,
        &markdown_output,
    )?;
    assert_eq!(
        std::fs::read_to_string(&markdown_output)?,
        render_image_storage_report_markdown(&report)
    );

    std::fs::write(&markdown_output, b"drifted")?;
    assert!(publish_image_storage_report(
        &fixture.plan,
        &fixture.after_inventory,
        &json_output,
        &markdown_output,
    )
    .is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn report_publication_rejects_symbolic_link_destinations() -> Result<()> {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new()?;
    fixture.create_plan()?;
    apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z")?;
    write_inventory(&fixture.after_inventory, &fixture, false)?;
    let redirect = fixture.plan.with_file_name("redirect.json");
    let json_output = fixture.plan.with_file_name("image-storage-report.json");
    let markdown_output = fixture.plan.with_file_name("image-storage-report.md");
    std::fs::write(&redirect, b"redirect")?;
    symlink(&redirect, &json_output)?;

    assert!(publish_image_storage_report(
        &fixture.plan,
        &fixture.after_inventory,
        &json_output,
        &markdown_output,
    )
    .is_err());
    assert_eq!(std::fs::read(&redirect)?, b"redirect");
    assert!(!markdown_output.exists());
    Ok(())
}

#[test]
fn plan_refuses_clobber_and_drifted_source_bytes() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.create_plan()?;
    assert!(fixture.create_plan().is_err());

    std::fs::write(&fixture.obsolete, b"changed")?;
    assert!(
        apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z").is_err()
    );
    let persisted = load_image_storage_plan(&fixture.plan)?;
    assert_eq!(persisted.state, ImageStoragePlanState::Applying);
    assert!(persisted.moves.iter().any(|row| row.applied));
    assert!(persisted.moves.iter().any(|row| !row.applied));

    std::fs::write(&fixture.obsolete, b"obsolete")?;
    let resumed = apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:01:00Z")?;
    assert_eq!(resumed.state, ImageStoragePlanState::Applied);
    assert!(resumed.moves.iter().all(|row| row.applied));
    Ok(())
}

#[test]
fn rollback_resumes_from_persisted_reverse_progress() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.create_plan()?;
    let applied = apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z")?;
    let detached = applied
        .moves
        .iter()
        .find(|row| row.source.ends_with("/detached-obsolete.sif"))
        .context("find detached image-storage move")?;
    std::fs::write(&detached.target, b"changed")?;

    assert!(rollback_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:01:00Z")
        .is_err());
    let interrupted = load_image_storage_plan(&fixture.plan)?;
    assert_eq!(interrupted.state, ImageStoragePlanState::RollingBack);
    assert!(interrupted.moves.iter().any(|row| !row.applied));

    std::fs::write(&detached.target, b"detached")?;
    let resumed =
        rollback_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:02:00Z")?;
    assert_eq!(resumed.state, ImageStoragePlanState::RolledBack);
    assert!(resumed.moves.iter().all(|row| !row.applied));
    Ok(())
}

struct Fixture {
    _temporary: tempfile::TempDir,
    study: PathBuf,
    images: PathBuf,
    inventory: PathBuf,
    after_inventory: PathBuf,
    plan: PathBuf,
    canonical: PathBuf,
    obsolete: PathBuf,
    interrupted: PathBuf,
    detached: PathBuf,
    historical: PathBuf,
    detached_historical: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self> {
        let temporary = tempfile::tempdir()?;
        let study = temporary.path().join("STUDY");
        let containers = study.join("containers");
        let images = containers.join("apptainer");
        let evidence = containers.join("evidence/inventory").join("a".repeat(40));
        let canonical = images.join("samtools-1.20-current.sif");
        let obsolete = images.join("samtools-1.19-obsolete.sif");
        let interrupted = images.join(".building/samtools.partial.sif");
        let detached = images.join("detached-obsolete.sif");
        let historical = images.join("samtools-1.18-historical.sif");
        let detached_historical = images.join("detached-historical.sif");
        std::fs::create_dir_all(interrupted.parent().context("resolve staged-image parent")?)?;
        std::fs::create_dir_all(&evidence)?;
        std::fs::write(&canonical, b"canonical")?;
        std::fs::write(&obsolete, b"obsolete")?;
        std::fs::write(&interrupted, b"interrupted")?;
        std::fs::write(&detached, b"detached")?;
        std::fs::write(&historical, b"historical")?;
        std::fs::write(&detached_historical, b"detached-historical")?;
        let fixture = Self {
            _temporary: temporary,
            study,
            images,
            inventory: evidence.join("image-inventory.json"),
            after_inventory: evidence.join("image-inventory-after-storage.json"),
            plan: evidence.join("image-storage-plan.json"),
            canonical,
            obsolete,
            interrupted,
            detached,
            historical,
            detached_historical,
        };
        write_inventory(&fixture.inventory, &fixture, true)?;
        Ok(fixture)
    }

    fn create_plan(&self) -> Result<bijux_dna_environment::api::ImageStoragePlan> {
        create_image_storage_plan(
            &ImageStoragePlanRequest {
                inventory_path: &self.inventory,
                image_root: &self.images,
                quarantine_root: &self.images.join(".quarantine").join("a".repeat(40)),
                study_root: &self.study,
                created_at_utc: "2026-08-05T07:59:00Z",
            },
            &self.plan,
        )
    }
}

fn write_inventory(path: &Path, fixture: &Fixture, include_obsolete: bool) -> Result<()> {
    let canonical_sha = bijux_dna_infra::hash_file_sha256(&fixture.canonical)?;
    let mut candidates = vec![ContainerImageCandidate {
        admitted: true,
        current_recipe: true,
        disposition: ImageCandidateDisposition::Canonical,
        path: locator(&fixture.study, &fixture.canonical),
        sha256: canonical_sha.clone(),
        size_bytes: std::fs::metadata(&fixture.canonical)?.len(),
        real_run_references: 0,
        run_references: Vec::new(),
        build_evidence: Vec::new(),
    }];
    if include_obsolete {
        candidates.push(ContainerImageCandidate {
            admitted: true,
            current_recipe: false,
            disposition: ImageCandidateDisposition::UnreferencedObsolete,
            path: locator(&fixture.study, &fixture.obsolete),
            sha256: bijux_dna_infra::hash_file_sha256(&fixture.obsolete)?,
            size_bytes: std::fs::metadata(&fixture.obsolete)?.len(),
            real_run_references: 0,
            run_references: Vec::new(),
            build_evidence: Vec::new(),
        });
        candidates.push(historical_tool_candidate(fixture)?);
    }
    let publication = ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2.to_string(),
        catalog_sha256: "b".repeat(64),
        repository_commit: "a".repeat(40),
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: "bijux.container.image_inventory_scan.v1".to_string(),
            raw_sha256: "c".repeat(64),
        },
        summary: ContainerImageInventorySummary {
            tool_count: 1,
            states: BTreeMap::from([("current".to_string(), 1)]),
            historical_dependency_count: if include_obsolete { 2 } else { 0 },
            unknown_image_count: u64::from(include_obsolete),
            staged_image_count: u64::from(include_obsolete),
            unreferenced_obsolete_count: if include_obsolete { 2 } else { 0 },
        },
        tools: vec![ContainerImageInventoryTool {
            tool: "samtools".to_string(),
            version: "1.20".to_string(),
            support_disposition: ContainerSupportDisposition::Retained,
            apptainer_supported: true,
            state: ContainerImageState::Current,
            selected_image_sha256: Some(canonical_sha),
            candidates,
        }],
        historical_images: if include_obsolete {
            vec![historical_detached_image(fixture)?]
        } else {
            Vec::new()
        },
        unknown_images: if include_obsolete {
            vec![ContainerDetachedImage {
                catalog_tool: Some("samtools".to_string()),
                disposition: DetachedImageDisposition::UnreferencedObsolete,
                path: locator(&fixture.study, &fixture.detached),
                sha256: bijux_dna_infra::hash_file_sha256(&fixture.detached)?,
                size_bytes: std::fs::metadata(&fixture.detached)?.len(),
                real_run_references: 0,
                run_references: Vec::new(),
            }]
        } else {
            Vec::new()
        },
        staged_images: if include_obsolete {
            vec![ContainerStagedImage {
                path: locator(&fixture.study, &fixture.interrupted),
                sha256: bijux_dna_infra::hash_file_sha256(&fixture.interrupted)?,
                size_bytes: std::fs::metadata(&fixture.interrupted)?.len(),
            }]
        } else {
            Vec::new()
        },
    };
    bijux_dna_infra::atomic_create_json(path, &publication)?;
    Ok(())
}

fn historical_tool_candidate(fixture: &Fixture) -> Result<ContainerImageCandidate> {
    Ok(ContainerImageCandidate {
        admitted: true,
        current_recipe: false,
        disposition: ImageCandidateDisposition::HistoricalRunDependency,
        path: locator(&fixture.study, &fixture.historical),
        sha256: bijux_dna_infra::hash_file_sha256(&fixture.historical)?,
        size_bytes: std::fs::metadata(&fixture.historical)?.len(),
        real_run_references: 1,
        run_references: vec![
            "repository://containers/runs/run-1000-samtools/evidence.lock.json".to_string()
        ],
        build_evidence: Vec::new(),
    })
}

fn historical_detached_image(fixture: &Fixture) -> Result<ContainerDetachedImage> {
    Ok(ContainerDetachedImage {
        catalog_tool: Some("samtools".to_string()),
        disposition: DetachedImageDisposition::HistoricalRunDependency,
        path: locator(&fixture.study, &fixture.detached_historical),
        sha256: bijux_dna_infra::hash_file_sha256(&fixture.detached_historical)?,
        size_bytes: std::fs::metadata(&fixture.detached_historical)?.len(),
        real_run_references: 1,
        run_references: vec![
            "repository://containers/runs/run-1001-samtools/evidence.lock.json".to_string()
        ],
    })
}

fn locator(study: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(study).map_or(path, |relative| relative);
    format!("study://{}", relative.display())
}
