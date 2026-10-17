use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    apply_image_storage_plan, create_image_storage_plan, derive_image_storage_report,
    image_storage_report_matches, load_image_storage_plan, publish_image_storage_report,
    render_image_storage_report_markdown, rollback_image_storage_plan, verify_image_storage_plan,
    ContainerArtifactDelivery, ContainerDetachedImage, ContainerImageCandidate,
    ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerProductSupport,
    ContainerStagedImage, DetachedImageDisposition, ImageCandidateDisposition,
    ImageInventoryPublicationIdentity, ImageStoragePlanRequest, ImageStoragePlanState,
    ImageStorageVerdict, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3,
};

#[test]
fn quarantine_plan_is_resumable_verified_and_reversible() -> Result<()> {
    let fixture = Fixture::new()?;
    let plan = fixture.create_plan()?;
    assert_eq!(plan.file_count, 6);

    let applied = apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:00:00Z")?;
    assert_eq!(applied.state, ImageStoragePlanState::Applied);
    verify_image_storage_plan(&fixture.plan, &fixture.images)?;
    assert!(!fixture.obsolete.exists());
    assert!(!fixture.interrupted.exists());
    assert!(!fixture.detached.exists());
    assert!(!fixture.historical.exists());
    assert!(!fixture.detached_historical.exists());
    assert!(!fixture.stale_selected.exists());

    let rolled_back =
        rollback_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:01:00Z")?;
    assert_eq!(rolled_back.state, ImageStoragePlanState::RolledBack);
    assert_eq!(std::fs::read(&fixture.obsolete)?, b"obsolete");
    assert_eq!(std::fs::read(&fixture.interrupted)?, b"interrupted");
    assert_eq!(std::fs::read(&fixture.detached)?, b"detached");
    assert_eq!(std::fs::read(&fixture.historical)?, b"historical");
    assert_eq!(std::fs::read(&fixture.detached_historical)?, b"detached-historical");
    assert_eq!(std::fs::read(&fixture.stale_selected)?, b"stale-selected");
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
    assert_eq!(report.moved_file_count, 6);
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
    assert!(image_storage_report_matches(
        &fixture.plan,
        &fixture.after_inventory,
        &json_output,
        &markdown_output,
    )?);
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
    assert!(!image_storage_report_matches(
        &fixture.plan,
        &fixture.after_inventory,
        &json_output,
        &markdown_output,
    )?);
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
    stale_selected: PathBuf,
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
        let stale_selected = images.join("bijux-dna-stale-selected.sif");
        std::fs::create_dir_all(interrupted.parent().context("resolve staged-image parent")?)?;
        std::fs::create_dir_all(&evidence)?;
        std::fs::write(&canonical, b"canonical")?;
        std::fs::write(&obsolete, b"obsolete")?;
        std::fs::write(&interrupted, b"interrupted")?;
        std::fs::write(&detached, b"detached")?;
        std::fs::write(&historical, b"historical")?;
        std::fs::write(&detached_historical, b"detached-historical")?;
        std::fs::write(&stale_selected, b"stale-selected")?;
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
            stale_selected,
        };
        write_inventory(&fixture.inventory, &fixture, true)?;
        Ok(fixture)
    }

    fn create_plan(&self) -> Result<bijux_dna_containers::api::ImageStoragePlan> {
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
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3.to_string(),
        catalog_sha256: "b".repeat(64),
        repository_commit: "a".repeat(40),
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: "bijux.container.image_inventory_scan.v1".to_string(),
            raw_sha256: "c".repeat(64),
        },
        summary: ContainerImageInventorySummary {
            tool_count: if include_obsolete { 3 } else { 2 },
            states: if include_obsolete {
                BTreeMap::from([
                    ("current".to_string(), 1),
                    ("historical".to_string(), 1),
                    ("not_applicable".to_string(), 1),
                ])
            } else {
                BTreeMap::from([("current".to_string(), 1), ("not_applicable".to_string(), 1)])
            },
            historical_dependency_count: if include_obsolete { 2 } else { 0 },
            unknown_image_count: u64::from(include_obsolete),
            staged_image_count: u64::from(include_obsolete),
            unreferenced_obsolete_count: if include_obsolete { 2 } else { 0 },
        },
        tools: [ContainerImageInventoryTool {
            tool: "samtools".to_string(),
            version: "1.20".to_string(),
            product_support: ContainerProductSupport::Supported,
            artifact_delivery: ContainerArtifactDelivery::RedistributableImage,
            redistribution_permitted: true,
            apptainer_supported: true,
            state: ContainerImageState::Current,
            selected_image_sha256: Some(canonical_sha),
            candidates,
        }]
        .into_iter()
        .chain(include_obsolete.then(|| stale_selected_tool(fixture)).transpose()?)
        .chain([external_artifact_tool()])
        .collect(),
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

fn external_artifact_tool() -> ContainerImageInventoryTool {
    ContainerImageInventoryTool {
        tool: "external".to_string(),
        version: "1.0.0".to_string(),
        product_support: ContainerProductSupport::Supported,
        artifact_delivery: ContainerArtifactDelivery::ExternalArtifact,
        redistribution_permitted: false,
        apptainer_supported: false,
        state: ContainerImageState::NotApplicable,
        selected_image_sha256: None,
        candidates: Vec::new(),
    }
}

fn stale_selected_tool(fixture: &Fixture) -> Result<ContainerImageInventoryTool> {
    let sha256 = bijux_dna_infra::hash_file_sha256(&fixture.stale_selected)?;
    Ok(ContainerImageInventoryTool {
        tool: "bijux_dna".to_string(),
        version: "0.1.0".to_string(),
        product_support: ContainerProductSupport::Supported,
        artifact_delivery: ContainerArtifactDelivery::RedistributableImage,
        redistribution_permitted: true,
        apptainer_supported: true,
        state: ContainerImageState::Historical,
        selected_image_sha256: Some(sha256.clone()),
        candidates: vec![ContainerImageCandidate {
            admitted: true,
            current_recipe: false,
            disposition: ImageCandidateDisposition::Canonical,
            path: locator(&fixture.study, &fixture.stale_selected),
            sha256,
            size_bytes: std::fs::metadata(&fixture.stale_selected)?.len(),
            real_run_references: 0,
            run_references: Vec::new(),
            build_evidence: Vec::new(),
        }],
    })
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

#[cfg(unix)]
#[path = "image_storage/inventory_observation.rs"]
mod inventory_observation;
#[cfg(unix)]
use inventory_observation::{write_inventory_child_request, InventoryCapture, InventoryWriter};

#[cfg(unix)]
#[test]
fn plan_inventory_identity_matches_captured_bytes_after_path_replacement() -> Result<()> {
    if write_inventory_child_request()? {
        return Ok(());
    }
    let fixture = Fixture::new()?;
    let captured = std::fs::read(&fixture.inventory)?;
    let mut writer = InventoryWriter::start(&fixture, &captured, InventoryCapture::Valid)?;
    let planned = fixture.create_plan();
    writer.finish()?;
    let plan = planned?;
    assert_eq!(plan.file_count, 6);
    assert_fixture_images_unchanged(&fixture)?;
    assert_eq!(plan.inventory.sha256, bijux_dna_infra::hash_bytes_sha256(&captured));
    assert_eq!(load_image_storage_plan(&fixture.plan)?, plan);

    let replacement = std::fs::read(&fixture.inventory)?;
    assert_ne!(captured, replacement);
    let fresh_output = fixture.plan.with_file_name("fresh-inventory-plan.json");
    let fresh = create_fixture_plan(&fixture, &fresh_output, "2026-08-05T08:00:00Z")?;
    assert_eq!(fresh.file_count, 0);
    assert_eq!(fresh.inventory.sha256, bijux_dna_infra::hash_bytes_sha256(&replacement));
    let persisted = std::fs::read(&fixture.plan)?;
    let refusal = apply_image_storage_plan(&fixture.plan, &fixture.images, "2026-08-05T08:01:00Z")
        .err()
        .context("replacement inventory cannot authorize captured moves")?;
    assert!(refusal.to_string().contains("source inventory changed"));
    assert_eq!(std::fs::read(&fixture.plan)?, persisted);
    assert_fixture_images_unchanged(&fixture)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn plan_inventory_refuses_malformed_capture_after_path_replacement() -> Result<()> {
    for (captured, scenario, context) in [
        (
            &b"{malformed\n"[..],
            InventoryCapture::MalformedJson,
            "parse image-inventory publication",
        ),
        (&b"\xff\xfe"[..], InventoryCapture::NonUtf8, "read image-inventory publication"),
    ] {
        let fixture = Fixture::new()?;
        let mut writer = InventoryWriter::start(&fixture, captured, scenario)?;
        let planned = fixture.create_plan();
        writer.finish()?;
        let refusal = planned.err().context("captured malformed input must remain rejected")?;
        assert!(refusal.to_string().contains(context), "{refusal:#}");
        assert!(refusal.to_string().contains(&fixture.inventory.display().to_string()));
        assert!(!fixture.plan.exists());
        bijux_dna_containers::api::load_image_inventory_publication(&fixture.inventory)?;
        let fresh = fixture.create_plan()?;
        assert_eq!(fresh.file_count, 0);
        assert_fixture_images_unchanged(&fixture)?;
    }
    Ok(())
}

#[test]
fn plan_inventory_digest_preserves_legacy_source_bytes() -> Result<()> {
    for schema in [
        "bijux.container.image_inventory_publication.v1",
        "bijux.container.image_inventory_publication.v2",
    ] {
        let fixture = Fixture::new()?;
        let mut legacy: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&fixture.inventory)?)?;
        legacy["schema_version"] = schema.into();
        let tools = legacy["tools"].as_array_mut().context("inventory tools")?;
        for tool in tools {
            let fields = tool.as_object_mut().context("inventory tool fields")?;
            let disposition = if fields["artifact_delivery"] == "redistributable_image" {
                "retained"
            } else {
                "unsupported"
            };
            fields.insert("support_disposition".into(), disposition.into());
            fields.remove("product_support");
            fields.remove("artifact_delivery");
            fields.remove("redistribution_permitted");
            if fields["state"] == "not_applicable" {
                fields.insert("state".into(), "unsupported".into());
            }
        }
        let states = legacy["summary"]["states"].as_object_mut().context("inventory states")?;
        let count = states.remove("not_applicable").context("legacy unsupported state count")?;
        states.insert("unsupported".into(), count);
        let captured = format!(" \n{}\n", serde_json::to_string_pretty(&legacy)?).into_bytes();
        std::fs::write(&fixture.inventory, &captured)?;
        let normalized =
            bijux_dna_containers::api::load_image_inventory_publication(&fixture.inventory)?;
        assert_eq!(normalized.schema_version, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3);
        let plan = fixture.create_plan()?;
        assert_eq!(plan.file_count, 6);
        assert_eq!(plan.inventory.sha256, bijux_dna_infra::hash_bytes_sha256(&captured));
        assert_ne!(
            plan.inventory.sha256,
            bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&normalized)?)
        );
        assert_fixture_images_unchanged(&fixture)?;
    }
    Ok(())
}

#[test]
fn plan_inventory_rejection_preserves_error_priority_and_existing_output() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut invalid_identity: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&fixture.inventory)?)?;
    invalid_identity["repository_commit"] = "invalid".into();
    let invalid_identity_bytes = serde_json::to_vec(&invalid_identity)?;
    let output = b"existing plan must remain intact";
    std::fs::write(&fixture.plan, output)?;
    std::fs::remove_file(&fixture.inventory)?;
    let refusal = create_fixture_plan(&fixture, &fixture.plan, "invalid")
        .err()
        .context("timestamp admission precedes inventory acquisition")?;
    assert!(refusal.to_string().contains("timestamp"));
    let missing = create_fixture_plan(&fixture, &fixture.plan, "2026-08-05T08:00:00Z")
        .err()
        .context("missing inventory remains an acquisition error")?;
    assert!(missing.to_string().contains("read image-inventory publication"));
    for (captured, context) in [
        (&b"\xff\xfe"[..], "read image-inventory publication"),
        (&b"{malformed"[..], "parse image-inventory publication"),
        (
            &b"{\"schema_version\":\"unknown\"}"[..],
            "unsupported image-inventory publication schema",
        ),
        (
            &b"{\"schema_version\":\"bijux.container.image_inventory_publication.v3\"}"[..],
            "parse image-inventory publication",
        ),
        (invalid_identity_bytes.as_slice(), "validate image-inventory publication"),
    ] {
        std::fs::write(&fixture.inventory, captured)?;
        let refusal = create_image_storage_plan(
            &ImageStoragePlanRequest {
                inventory_path: &fixture.inventory,
                image_root: &fixture.images.join("missing"),
                quarantine_root: &fixture.images.join(".quarantine"),
                study_root: &fixture.study.join("missing"),
                created_at_utc: "2026-08-05T08:00:00Z",
            },
            &fixture.plan,
        )
        .err()
        .context("captured source refusal precedes root/output handling")?;
        assert!(refusal.to_string().contains(context), "{refusal:#}");
        assert_eq!(std::fs::read(&fixture.plan)?, output);
        assert_fixture_images_unchanged(&fixture)?;
    }
    Ok(())
}

fn create_fixture_plan(
    fixture: &Fixture,
    output: &Path,
    timestamp: &str,
) -> Result<bijux_dna_containers::api::ImageStoragePlan> {
    create_image_storage_plan(
        &ImageStoragePlanRequest {
            inventory_path: &fixture.inventory,
            image_root: &fixture.images,
            quarantine_root: &fixture.images.join(".quarantine").join("a".repeat(40)),
            study_root: &fixture.study,
            created_at_utc: timestamp,
        },
        output,
    )
}

fn assert_fixture_images_unchanged(fixture: &Fixture) -> Result<()> {
    for (path, bytes) in [
        (&fixture.canonical, &b"canonical"[..]),
        (&fixture.obsolete, &b"obsolete"[..]),
        (&fixture.interrupted, &b"interrupted"[..]),
        (&fixture.detached, &b"detached"[..]),
        (&fixture.historical, &b"historical"[..]),
        (&fixture.detached_historical, &b"detached-historical"[..]),
        (&fixture.stale_selected, &b"stale-selected"[..]),
    ] {
        assert_eq!(std::fs::read(path)?, bytes);
    }
    Ok(())
}
