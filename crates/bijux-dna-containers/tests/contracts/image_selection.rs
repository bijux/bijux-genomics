use std::collections::BTreeMap;

use bijux_dna_containers::evidence::{
    resolve_catalog_locked_apptainer_image, ContainerArtifactDelivery, ContainerCatalog,
    ContainerImageCandidate, ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerSupportDisposition,
    ImageCandidateDisposition, ImageInventoryPublicationIdentity,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
};

fn publication(image_sha256: String, size_bytes: u64) -> ContainerImageInventoryPublication {
    ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1.to_string(),
        catalog_sha256: "a".repeat(64),
        repository_commit: "b".repeat(40),
        publication: ImageInventoryPublicationIdentity {
            raw_schema_version: "bijux.container.image_inventory_scan.v1".to_string(),
            raw_sha256: "c".repeat(64),
        },
        summary: ContainerImageInventorySummary {
            tool_count: 1,
            states: BTreeMap::from([("current".to_string(), 1)]),
            historical_dependency_count: 0,
            unknown_image_count: 0,
            staged_image_count: 0,
            unreferenced_obsolete_count: 0,
        },
        tools: vec![ContainerImageInventoryTool {
            tool: "seqkit".to_string(),
            version: "2.12.0".to_string(),
            support_disposition: ContainerSupportDisposition::Retained,
            apptainer_supported: true,
            state: ContainerImageState::Current,
            selected_image_sha256: Some(image_sha256.clone()),
            candidates: vec![ContainerImageCandidate {
                admitted: true,
                current_recipe: true,
                disposition: ImageCandidateDisposition::Canonical,
                path: "study://containers/apptainer/seqkit.sif".to_string(),
                sha256: image_sha256,
                size_bytes,
                real_run_references: 0,
                run_references: Vec::new(),
                build_evidence: Vec::new(),
            }],
        }],
        historical_images: Vec::new(),
        unknown_images: Vec::new(),
        staged_images: Vec::new(),
    }
}

fn catalog() -> anyhow::Result<ContainerCatalog> {
    Ok(serde_json::from_value(serde_json::json!({
        "schema_version": "bijux.container.catalog.v1",
        "owner": "test",
        "contact": "test@example.com",
        "repository": "https://example.com/repository",
        "apptainer_architecture": "linux/amd64",
        "docker_architecture": "linux/arm64",
        "evidence_schema": "bijux.container.evidence.v1",
        "recipe_license": "GPL-3.0-only",
        "required_checks": ["version"],
        "evidence_dimensions": ["runtime"],
        "tools": [{
            "id": "seqkit",
            "display_name": "SeqKit",
            "purpose": "test",
            "version": "2.12.0",
            "support_disposition": "retained",
            "upstream": "https://example.com/seqkit",
            "documentation": "https://example.com/seqkit/docs",
            "license": "MIT",
            "domains": ["fastq"],
            "stage_ids": ["fastq.inspect"],
            "expected_bin": "seqkit",
            "version_command": "seqkit version",
            "help_command": "seqkit --help",
            "expected_version_regex": "^2[.]12[.]0$",
            "apptainer_definition": "containers/apptainer/seqkit.def",
            "apptainer_supported": true,
            "dockerfile": "",
            "docker_arm64_supported": false,
            "docker_arm64_reason": "test fixture",
            "build_network": true,
            "runtime_network": false,
            "real_test_stage": "fastq.inspect",
            "real_test_input_kind": "fastq"
        }]
    }))?)
}

#[test]
fn current_image_resolution_rechecks_absolute_file_identity() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("current-image-selection");
    let image = temporary.path().join("containers/apptainer/seqkit.sif");
    let parent = image.parent().ok_or_else(|| anyhow::anyhow!("test image has no parent"))?;
    std::fs::create_dir_all(parent)?;
    std::fs::write(&image, b"immutable seqkit image")?;
    let digest = bijux_dna_infra::hash_file_sha256(&image)?;
    let inventory = publication(digest.clone(), std::fs::metadata(&image)?.len());

    let selected = resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog()?,
        &inventory.repository_commit,
        temporary.path(),
        "seqkit",
    )?;

    assert_eq!(selected.tool_id, "seqkit");
    assert_eq!(selected.path, image.canonicalize()?);
    assert_eq!(selected.sha256, digest);
    assert_eq!(selected.repository_commit, "b".repeat(40));
    assert_eq!(selected.catalog_sha256, "a".repeat(64));
    Ok(())
}

#[test]
fn catalog_locked_resolution_rejects_stale_and_unselectable_authorities() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("catalog-locked-image-selection");
    let image = temporary.path().join("containers/apptainer/seqkit.sif");
    let parent = image.parent().ok_or_else(|| anyhow::anyhow!("test image has no parent"))?;
    std::fs::create_dir_all(parent)?;
    std::fs::write(&image, b"immutable seqkit image")?;
    let digest = bijux_dna_infra::hash_file_sha256(&image)?;
    let inventory = publication(digest, std::fs::metadata(&image)?.len());
    let catalog = catalog()?;

    resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog,
        &inventory.repository_commit,
        temporary.path(),
        "seqkit",
    )?;
    assert!(resolve_catalog_locked_apptainer_image(
        &inventory,
        &"d".repeat(64),
        &catalog,
        &inventory.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());
    assert!(resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog,
        &"e".repeat(40),
        temporary.path(),
        "seqkit",
    )
    .is_err());
    assert!(resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog,
        &inventory.repository_commit,
        temporary.path(),
        "unknown",
    )
    .is_err());

    let mut duplicate = inventory.clone();
    duplicate.tools.push(duplicate.tools[0].clone());
    duplicate.summary.tool_count = 2;
    duplicate.summary.states.insert("current".to_owned(), 2);
    assert!(resolve_catalog_locked_apptainer_image(
        &duplicate,
        &duplicate.catalog_sha256,
        &catalog,
        &duplicate.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());

    let mut unsupported = inventory.clone();
    unsupported.tools[0].support_disposition = ContainerSupportDisposition::Unsupported;
    unsupported.tools[0].apptainer_supported = false;
    unsupported.tools[0].state = ContainerImageState::Unsupported;
    unsupported.tools[0].selected_image_sha256 = None;
    unsupported.tools[0].candidates.clear();
    unsupported.summary.states = BTreeMap::from([("unsupported".to_owned(), 1)]);
    let mut unsupported_catalog = catalog.clone();
    unsupported_catalog.tools[0].artifact_delivery = ContainerArtifactDelivery::ExternalArtifact;
    unsupported_catalog.tools[0].redistribution_permitted = false;
    unsupported_catalog.tools[0].apptainer_supported = false;
    assert!(resolve_catalog_locked_apptainer_image(
        &unsupported,
        &unsupported.catalog_sha256,
        &unsupported_catalog,
        &unsupported.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());

    let mut ambiguous = inventory.clone();
    ambiguous.tools[0].state = ContainerImageState::Ambiguous;
    ambiguous.tools[0].selected_image_sha256 = None;
    ambiguous.tools[0].candidates[0].disposition = ImageCandidateDisposition::UnreferencedObsolete;
    let mut second = ambiguous.tools[0].candidates[0].clone();
    second.path = "study://containers/apptainer/seqkit-second.sif".to_owned();
    second.sha256 = "f".repeat(64);
    ambiguous.tools[0].candidates.push(second);
    ambiguous.summary.states = BTreeMap::from([("ambiguous".to_owned(), 1)]);
    ambiguous.summary.unreferenced_obsolete_count = 2;
    assert!(resolve_catalog_locked_apptainer_image(
        &ambiguous,
        &ambiguous.catalog_sha256,
        &catalog,
        &ambiguous.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());
    Ok(())
}

#[test]
fn current_image_resolution_rejects_changed_bytes_and_noncurrent_state() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("current-image-drift");
    let image = temporary.path().join("containers/apptainer/seqkit.sif");
    let parent = image.parent().ok_or_else(|| anyhow::anyhow!("test image has no parent"))?;
    std::fs::create_dir_all(parent)?;
    std::fs::write(&image, b"immutable seqkit image")?;
    let digest = bijux_dna_infra::hash_file_sha256(&image)?;
    let mut inventory = publication(digest, std::fs::metadata(&image)?.len());

    std::fs::write(&image, b"modified seqkit image")?;
    assert!(resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog()?,
        &inventory.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());

    inventory.tools[0].state = ContainerImageState::Historical;
    assert!(resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog()?,
        &inventory.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn current_image_resolution_rejects_symlinked_selection() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let temporary = bijux_dna_testkit::tempdir_for("current-image-symlink");
    let target = temporary.path().join("actual.sif");
    let image = temporary.path().join("containers/apptainer/seqkit.sif");
    let parent = image.parent().ok_or_else(|| anyhow::anyhow!("test image has no parent"))?;
    std::fs::create_dir_all(parent)?;
    std::fs::write(&target, b"immutable seqkit image")?;
    symlink(&target, &image)?;
    let digest = bijux_dna_infra::hash_file_sha256(&target)?;
    let inventory = publication(digest, std::fs::metadata(&target)?.len());

    assert!(resolve_catalog_locked_apptainer_image(
        &inventory,
        &inventory.catalog_sha256,
        &catalog()?,
        &inventory.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .is_err());
    Ok(())
}
