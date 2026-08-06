use std::collections::BTreeMap;

use bijux_dna_containers::evidence::{
    resolve_current_apptainer_image, ContainerImageCandidate, ContainerImageInventoryPublication,
    ContainerImageInventorySummary, ContainerImageInventoryTool, ContainerImageState,
    ContainerSupportDisposition, ImageCandidateDisposition, ImageInventoryPublicationIdentity,
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

#[test]
fn current_image_resolution_rechecks_absolute_file_identity() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("current-image-selection");
    let image = temporary.path().join("containers/apptainer/seqkit.sif");
    let parent = image.parent().ok_or_else(|| anyhow::anyhow!("test image has no parent"))?;
    std::fs::create_dir_all(parent)?;
    std::fs::write(&image, b"immutable seqkit image")?;
    let digest = bijux_dna_infra::hash_file_sha256(&image)?;
    let inventory = publication(digest.clone(), std::fs::metadata(&image)?.len());

    let selected = resolve_current_apptainer_image(&inventory, temporary.path(), "seqkit")?;

    assert_eq!(selected.tool_id, "seqkit");
    assert_eq!(selected.path, image.canonicalize()?);
    assert_eq!(selected.sha256, digest);
    assert_eq!(selected.repository_commit, "b".repeat(40));
    assert_eq!(selected.catalog_sha256, "a".repeat(64));
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
    assert!(resolve_current_apptainer_image(&inventory, temporary.path(), "seqkit").is_err());

    inventory.tools[0].state = ContainerImageState::Historical;
    assert!(resolve_current_apptainer_image(&inventory, temporary.path(), "seqkit").is_err());
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

    assert!(resolve_current_apptainer_image(&inventory, temporary.path(), "seqkit").is_err());
    Ok(())
}
