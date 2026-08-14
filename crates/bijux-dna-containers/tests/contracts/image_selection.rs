use std::collections::BTreeMap;
use std::process::Command;

use bijux_dna_containers::evidence::{
    resolve_catalog_locked_apptainer_artifact, resolve_catalog_locked_apptainer_image,
    ContainerArtifactDelivery, ContainerCatalog, ContainerImageCandidate,
    ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerProductSupport,
    EvidenceSourceIdentity, ExternalArtifactAdmission, ExternalArtifactAdmissionStatus,
    ExternalArtifactIdentity, ExternalArtifactKind, ExternalArtifactVersionProbe,
    ImageCandidateDisposition, ImageInventoryPublicationIdentity,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2,
};

fn publication(image_sha256: String, size_bytes: u64) -> ContainerImageInventoryPublication {
    ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3.to_string(),
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
            product_support: ContainerProductSupport::Supported,
            artifact_delivery: ContainerArtifactDelivery::RedistributableImage,
            redistribution_permitted: true,
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
            "product_support": "supported",
            "artifact_delivery": "redistributable_image",
            "redistribution_permitted": true,
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
fn delivery_aware_resolution_reverifies_external_apptainer_authority() -> anyhow::Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("external-sif-selection");
    let repository = temporary.path().join("repository");
    let study = temporary.path().join("study");
    let current_images = study.join("containers/current-images");
    std::fs::create_dir_all(repository.join("containers/contracts"))?;
    std::fs::create_dir_all(&current_images)?;

    let mut external_catalog = catalog()?;
    let tool = &mut external_catalog.tools[0];
    tool.artifact_delivery = ContainerArtifactDelivery::ExternalArtifact;
    tool.redistribution_permitted = false;
    tool.delivery_reason = "operator-provided licensed artifact".to_string();
    tool.external_artifact_supported = true;
    tool.external_artifact_kinds = vec!["apptainer_sif".to_string(), "binary".to_string()];
    tool.apptainer_definition.clear();
    tool.apptainer_supported = false;
    std::fs::write(
        repository.join("containers/contracts/catalog.toml"),
        toml::to_string_pretty(&external_catalog)?,
    )?;
    run_git(&repository, &["init", "--quiet"])?;
    run_git(&repository, &["config", "user.email", "fixture@example.invalid"])?;
    run_git(&repository, &["config", "user.name", "Fixture"])?;
    run_git(&repository, &["add", "containers/contracts/catalog.toml"])?;
    run_git(&repository, &["commit", "--quiet", "-m", "test: add external catalog"])?;
    let repository_commit = git_stdout(&repository, &["rev-parse", "HEAD"])?;
    let catalog_sha256 =
        bijux_dna_infra::hash_file_sha256(&repository.join("containers/contracts/catalog.toml"))?;

    let artifact = temporary.path().join("private-seqkit.sif");
    std::fs::write(&artifact, b"private immutable seqkit SIF")?;
    let artifact = artifact.canonicalize()?;
    let artifact_sha256 = bijux_dna_infra::hash_file_sha256(&artifact)?;
    let version_output = "2.12.0";
    let admission = ExternalArtifactAdmission {
        schema_version: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2.to_string(),
        availability: ExternalArtifactAdmissionStatus::Admitted,
        recorded_at_utc: "2026-08-14T22:20:00Z".to_string(),
        repository_commit: repository_commit.clone(),
        catalog: EvidenceSourceIdentity {
            path: "containers/contracts/catalog.toml".to_string(),
            sha256: catalog_sha256.clone(),
        },
        distributable_image_root: current_images.canonicalize()?.display().to_string(),
        tool_id: "seqkit".to_string(),
        version: "2.12.0".to_string(),
        product_support: ContainerProductSupport::Supported,
        artifact_delivery: ContainerArtifactDelivery::ExternalArtifact,
        delivery_reason: "operator-provided licensed artifact".to_string(),
        stage_ids: vec!["fastq.inspect".to_string()],
        expected_binary: "seqkit".to_string(),
        version_command: "seqkit version".to_string(),
        expected_version_regex: "^2[.]12[.]0$".to_string(),
        redistribution_permitted: false,
        lawful_access_basis: Some("institutional license".to_string()),
        artifact: Some(ExternalArtifactIdentity {
            kind: ExternalArtifactKind::ApptainerSif,
            path: artifact.display().to_string(),
            sha256: artifact_sha256.clone(),
            size_bytes: std::fs::metadata(&artifact)?.len(),
        }),
        version_probe: Some(ExternalArtifactVersionProbe {
            command: vec![
                "apptainer".to_string(),
                "exec".to_string(),
                artifact.display().to_string(),
                "seqkit".to_string(),
                "version".to_string(),
            ],
            output: version_output.to_string(),
            output_sha256: bijux_dna_infra::hash_bytes_sha256(version_output.as_bytes()),
        }),
        blocker_reason: None,
    };
    let admission_root =
        study.join("containers/evidence/external-artifacts").join(&repository_commit);
    std::fs::create_dir_all(&admission_root)?;
    std::fs::write(admission_root.join("seqkit.json"), serde_json::to_vec_pretty(&admission)?)?;

    let mut inventory = publication("d".repeat(64), 1);
    inventory.catalog_sha256.clone_from(&catalog_sha256);
    inventory.repository_commit.clone_from(&repository_commit);
    inventory.tools[0].artifact_delivery = ContainerArtifactDelivery::ExternalArtifact;
    inventory.tools[0].redistribution_permitted = false;
    inventory.tools[0].apptainer_supported = false;
    inventory.tools[0].state = ContainerImageState::NotApplicable;
    inventory.tools[0].selected_image_sha256 = None;
    inventory.tools[0].candidates.clear();
    inventory.summary.states = BTreeMap::from([("not_applicable".to_string(), 1)]);

    let selected = resolve_catalog_locked_apptainer_artifact(
        &inventory,
        &catalog_sha256,
        &external_catalog,
        &repository_commit,
        &repository,
        &study,
        "seqkit",
    )?;
    assert_eq!(selected.path, artifact);
    assert_eq!(selected.sha256, artifact_sha256);
    assert_eq!(selected.repository_commit, repository_commit);

    let mut blocked = admission.clone();
    blocked.availability = ExternalArtifactAdmissionStatus::BlockedExternalAccess;
    blocked.lawful_access_basis = None;
    blocked.artifact = None;
    blocked.version_probe = None;
    blocked.blocker_reason = Some("licensed artifact not supplied".to_string());
    std::fs::write(admission_root.join("seqkit.json"), serde_json::to_vec_pretty(&blocked)?)?;
    let blocked_error = resolve_catalog_locked_apptainer_artifact(
        &inventory,
        &catalog_sha256,
        &external_catalog,
        &repository_commit,
        &repository,
        &study,
        "seqkit",
    )
    .expect_err("blocked external access must stop before execution");
    assert_eq!(
        blocked_error.to_string(),
        "external artifact access blocked for seqkit: licensed artifact not supplied"
    );

    let mut binary = admission;
    binary.artifact.as_mut().expect("fixture artifact").kind = ExternalArtifactKind::Binary;
    binary.version_probe.as_mut().expect("fixture version probe").command =
        vec![artifact.display().to_string(), "version".to_string()];
    std::fs::write(admission_root.join("seqkit.json"), serde_json::to_vec_pretty(&binary)?)?;
    let binary_error = resolve_catalog_locked_apptainer_artifact(
        &inventory,
        &catalog_sha256,
        &external_catalog,
        &repository_commit,
        &repository,
        &study,
        "seqkit",
    )
    .expect_err("binary admission must use command-binding authority");
    assert_eq!(
        binary_error.to_string(),
        "admitted external artifact for seqkit is a binary and requires command-binding authority"
    );
    Ok(())
}

fn run_git(repository: &std::path::Path, arguments: &[&str]) -> anyhow::Result<()> {
    let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
    anyhow::ensure!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn git_stdout(repository: &std::path::Path, arguments: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
    anyhow::ensure!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
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

    let mut external = inventory.clone();
    external.tools[0].artifact_delivery = ContainerArtifactDelivery::ExternalArtifact;
    external.tools[0].redistribution_permitted = false;
    external.tools[0].apptainer_supported = false;
    external.tools[0].state = ContainerImageState::NotApplicable;
    external.tools[0].selected_image_sha256 = None;
    external.tools[0].candidates.clear();
    external.summary.states = BTreeMap::from([("not_applicable".to_owned(), 1)]);
    let mut external_catalog = catalog.clone();
    external_catalog.tools[0].artifact_delivery = ContainerArtifactDelivery::ExternalArtifact;
    external_catalog.tools[0].redistribution_permitted = false;
    external_catalog.tools[0].apptainer_supported = false;
    let error = resolve_catalog_locked_apptainer_image(
        &external,
        &external.catalog_sha256,
        &external_catalog,
        &external.repository_commit,
        temporary.path(),
        "seqkit",
    )
    .expect_err("external artifact must resolve through its admission authority");
    assert_eq!(
        error.to_string(),
        "tool seqkit requires admitted external_artifact execution authority"
    );

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
