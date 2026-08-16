use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use bijux_dna_containers::evidence::{
    apptainer_recipe_identity_at_commit, derive_image_inventory_evidence,
    derive_image_inventory_publication, image_inventory_request_from_activation_plan,
    publish_image_inventory_evidence, repository_head_commit, ContainerImageState,
    DetachedImageDisposition, EvidenceSourceIdentity, ImageActivationPlan,
    ImageActivationPlanState, ImageActivationPublication, ImageCandidateDisposition,
    ImageInventoryDerivationRequest, ImageInventoryEvidenceRoots, IMAGE_ACTIVATION_PLAN_SCHEMA_V1,
};
use serde_json::json;
use tempfile::TempDir;

struct Fixture {
    _temporary: TempDir,
    repository: PathBuf,
    study: PathBuf,
    request: ImageInventoryDerivationRequest,
}

impl Fixture {
    fn current() -> Result<Self> {
        let temporary = tempfile::tempdir()?;
        let repository = temporary.path().join("repository");
        let study = temporary.path().join("study");
        fs::create_dir_all(repository.join("containers/contracts"))?;
        fs::create_dir_all(repository.join("containers/apptainer"))?;
        fs::create_dir_all(repository.join("containers/tools/example"))?;
        fs::create_dir_all(study.join("containers/apptainer"))?;
        fs::create_dir_all(study.join("containers/evidence"))?;
        fs::create_dir_all(repository.join("containers/runs"))?;
        fs::write(repository.join("containers/contracts/catalog.toml"), catalog_toml())?;
        fs::write(
            repository.join("containers/apptainer/example.def"),
            "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n",
        )?;
        fs::write(repository.join("containers/tools/example/example"), "#!/bin/sh\n")?;
        git(&repository, &["init", "-q"])?;
        git(&repository, &["config", "user.name", "Bijux Test"])?;
        git(&repository, &["config", "user.email", "test@bijux.io"])?;
        git(&repository, &["add", "containers"])?;
        git(&repository, &["commit", "-qm", "add governed container"])?;

        let request = ImageInventoryDerivationRequest {
            repository_root: repository.clone(),
            study_root: study.clone(),
            evidence_roots: vec![ImageInventoryEvidenceRoots {
                image_root: study.join("containers/apptainer"),
                build_evidence_root: study.join("containers/evidence"),
            }],
            run_evidence_root: repository.join("containers/runs"),
        };
        let fixture = Self { _temporary: temporary, repository, study, request };
        fixture.add_admitted_image(b"current immutable image")?;
        Ok(fixture)
    }

    fn add_admitted_image(&self, bytes: &[u8]) -> Result<String> {
        self.add_admitted_image_to(
            bytes,
            &self.study.join("containers/apptainer"),
            &self.study.join("containers/evidence"),
        )
    }

    fn add_admitted_image_to(
        &self,
        bytes: &[u8],
        image_root: &Path,
        build_evidence_root: &Path,
    ) -> Result<String> {
        let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
        let filename = format!("example-1.0.0-linux-amd64-{digest:.12}.sif");
        fs::create_dir_all(image_root)?;
        fs::create_dir_all(build_evidence_root)?;
        let image = image_root.join(&filename);
        fs::write(&image, bytes)?;
        let commit = repository_head_commit(&self.repository)?;
        let recipe = apptainer_recipe_identity_at_commit(
            &self.repository,
            &commit,
            "containers/apptainer/example.def",
        )?;
        let evidence = json!({
            "schema_version": "bijux.container.build_evidence.v1",
            "tool": "example",
            "version": "1.0.0",
            "status": "passed",
            "repository_commit": commit,
            "definition": {
                "path": "containers/apptainer/example.def",
                "sha256": recipe.definition_sha256,
            },
            "build_context": recipe.build_context,
            "image": {
                "path": image,
                "sha256": digest,
                "size_bytes": bytes.len(),
            },
        });
        fs::write(
            build_evidence_root.join(format!("{digest}.build.json")),
            serde_json::to_vec_pretty(&evidence)?,
        )?;
        Ok(digest)
    }

    fn reference_image_from_run(&self, digest: &str) -> Result<()> {
        let run_root = self.repository.join("containers/runs/run-2000-example");
        fs::create_dir_all(&run_root)?;
        fs::write(
            run_root.join("evidence.lock.json"),
            serde_json::to_vec_pretty(&json!({
                "schema_version": "bijux.container.run_evidence_lock.v1",
                "image": {"sha256": digest},
            }))?,
        )?;
        Ok(())
    }

    fn rewrite_build_evidence_as_flattened(&self, digest: &str) -> Result<()> {
        let evidence_path =
            self.study.join("containers/evidence").join(format!("{digest}.build.json"));
        let nested: serde_json::Value = serde_json::from_slice(&fs::read(&evidence_path)?)?;
        let flattened = json!({
            "schema_version": nested["schema_version"],
            "tool": nested["tool"],
            "version": nested["version"],
            "status": nested["status"],
            "repository_commit": nested["repository_commit"],
            "definition_sha256": nested["definition"]["sha256"],
            "image_path": nested["image"]["path"],
            "image_sha256": nested["image"]["sha256"],
            "image_size_bytes": nested["image"]["size_bytes"],
        });
        fs::write(evidence_path, serde_json::to_vec_pretty(&flattened)?)?;
        Ok(())
    }
}

#[test]
fn owner_immutably_publishes_the_scan_and_inventory() -> Result<()> {
    let fixture = Fixture::current()?;
    let scan = fixture.study.join("publications/scan.json");
    let publication = fixture.study.join("publications/inventory.json");

    let evidence = publish_image_inventory_evidence(&fixture.request, &scan, &publication)?;

    let persisted_scan: serde_json::Value = serde_json::from_slice(&fs::read(scan)?)?;
    let persisted_publication: serde_json::Value = serde_json::from_slice(&fs::read(publication)?)?;
    assert_eq!(persisted_scan, serde_json::to_value(evidence.scan)?);
    assert_eq!(persisted_publication, serde_json::to_value(evidence.publication)?);
    Ok(())
}

#[test]
fn owner_rejects_overlapping_or_existing_inventory_publications() -> Result<()> {
    let fixture = Fixture::current()?;
    let retained = fixture.study.join("publications/retained.json");
    fs::create_dir_all(retained.parent().context("retained parent")?)?;
    fs::write(&retained, b"retained\n")?;
    let other = fixture.study.join("publications/other.json");

    assert!(publish_image_inventory_evidence(&fixture.request, &retained, &retained).is_err());
    assert!(publish_image_inventory_evidence(&fixture.request, &retained, &other).is_err());
    assert_eq!(fs::read(retained)?, b"retained\n");
    assert!(!other.exists());
    Ok(())
}

#[test]
fn applied_activation_reconstructs_exact_campaign_evidence_pairs() -> Result<()> {
    let fixture = Fixture::current()?;
    let staging_root = fixture.study.join("containers/staging");
    let selected = Path::new("campaign-a/selected-one");
    let source_root = staging_root.join(selected);
    let build_root = fixture.study.join("containers/evidence/image-campaigns").join(selected);
    fs::create_dir_all(&source_root)?;
    fs::create_dir_all(&build_root)?;
    let bytes = b"activated immutable image";
    let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
    let filename = format!("example-1.0.0-linux-amd64-{digest:.12}.sif");
    let source = source_root.join(&filename);
    let target = fixture.study.join("containers/apptainer").join(&filename);
    fs::write(&source, bytes)?;
    fs::hard_link(&source, &target)?;

    let inventory = fixture.study.join("activation-inventory.json");
    fs::write(
        &inventory,
        serde_json::to_vec_pretty(&derive_image_inventory_publication(&fixture.request)?)?,
    )?;
    let plan = ImageActivationPlan {
        schema_version: IMAGE_ACTIVATION_PLAN_SCHEMA_V1.to_string(),
        state: ImageActivationPlanState::Applied,
        created_at_utc: "2026-08-15T12:00:00Z".to_string(),
        inventory: EvidenceSourceIdentity {
            path: inventory.display().to_string(),
            sha256: bijux_dna_infra::hash_file_sha256(&inventory)?,
        },
        source_root: staging_root.canonicalize()?.display().to_string(),
        active_root: fixture
            .study
            .join("containers/apptainer")
            .canonicalize()?
            .display()
            .to_string(),
        file_count: 1,
        size_bytes: u64::try_from(bytes.len())?,
        publications: vec![ImageActivationPublication {
            tool: "example".to_string(),
            source: source.canonicalize()?.display().to_string(),
            target: target.canonicalize()?.display().to_string(),
            size_bytes: u64::try_from(bytes.len())?,
            sha256: digest,
            applied: true,
            applied_at_utc: Some("2026-08-15T12:01:00Z".to_string()),
        }],
        completed_at_utc: Some("2026-08-15T12:01:00Z".to_string()),
        rolled_back_at_utc: None,
    };
    let plan_path = fixture.study.join("activation-plan.json");
    fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;

    let request = image_inventory_request_from_activation_plan(
        &fixture.repository,
        &fixture.study,
        &plan_path,
        &fixture.repository.join("containers/runs"),
    )?;

    assert_eq!(request.evidence_roots.len(), 1);
    assert_eq!(
        request.evidence_roots[0].image_root,
        fixture.study.join("containers/apptainer").canonicalize()?
    );
    assert_eq!(request.evidence_roots[0].build_evidence_root, build_root.canonicalize()?);
    Ok(())
}

#[test]
fn composes_current_single_tool_campaign_with_prior_campaign_evidence() -> Result<()> {
    let fixture = Fixture::current()?;
    let prior_digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n\n%post\n  true\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "change example recipe"])?;
    let current_images = fixture.study.join("containers/staging/current");
    let current_evidence = fixture.study.join("containers/evidence/image-campaigns/current");
    let current_digest = fixture.add_admitted_image_to(
        b"current single-tool campaign image",
        &current_images,
        &current_evidence,
    )?;
    let mut request = fixture.request.clone();
    request.evidence_roots.push(ImageInventoryEvidenceRoots {
        image_root: current_images,
        build_evidence_root: current_evidence,
    });

    let evidence = derive_image_inventory_evidence(&request)?;
    let tool = evidence.publication.tools.first().context("publication omits example")?;

    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(current_digest.as_str()));
    assert!(tool.candidates.iter().any(|candidate| {
        candidate.sha256 == prior_digest
            && candidate.disposition == ImageCandidateDisposition::UnreferencedObsolete
    }));
    assert_eq!(evidence.publication.summary.unreferenced_obsolete_count, 1);
    assert_eq!(evidence.scan.images.len(), 2);
    assert_eq!(evidence.scan.build_evidence.len(), 2);
    assert_eq!(evidence.scan.evidence_roots.len(), 2);
    assert_eq!(evidence.scan.evidence_roots[1].image_root, "study://containers/staging/current");
    assert_eq!(
        evidence.scan.evidence_roots[1].build_evidence_root,
        "study://containers/evidence/image-campaigns/current"
    );
    Ok(())
}

#[test]
fn composes_multiple_campaigns_for_one_activated_image_root() -> Result<()> {
    let fixture = Fixture::current()?;
    let prior_evidence = fixture.study.join("containers/evidence/prior");
    fs::create_dir_all(&prior_evidence)?;
    let evidence_path = fs::read_dir(fixture.study.join("containers/evidence"))?
        .find_map(|entry| {
            entry.ok().and_then(|entry| {
                entry.file_type().ok().filter(std::fs::FileType::is_file).map(|_| entry.path())
            })
        })
        .context("fixture omits prior build evidence")?;
    fs::rename(
        &evidence_path,
        prior_evidence.join(evidence_path.file_name().context("evidence has no filename")?),
    )?;
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n\n%post\n  true\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "change example recipe"])?;
    let current_evidence = fixture.study.join("containers/evidence/current");
    let current_digest = fixture.add_admitted_image_to(
        b"current activated campaign image",
        &fixture.study.join("containers/apptainer"),
        &current_evidence,
    )?;
    let request = ImageInventoryDerivationRequest {
        repository_root: fixture.repository.clone(),
        study_root: fixture.study.clone(),
        evidence_roots: vec![
            ImageInventoryEvidenceRoots {
                image_root: fixture.study.join("containers/apptainer"),
                build_evidence_root: prior_evidence,
            },
            ImageInventoryEvidenceRoots {
                image_root: fixture.study.join("containers/apptainer"),
                build_evidence_root: current_evidence,
            },
        ],
        run_evidence_root: fixture.repository.join("containers/runs"),
    };

    let evidence = derive_image_inventory_evidence(&request)?;
    let tool = evidence.publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(current_digest.as_str()));
    assert_eq!(tool.candidates.len(), 2);
    assert_eq!(evidence.scan.images.len(), 2);
    assert_eq!(evidence.scan.build_evidence.len(), 2);
    assert_eq!(evidence.scan.evidence_roots.len(), 2);
    Ok(())
}

#[test]
fn activated_roster_ignores_superseded_builds_absent_from_active_images() -> Result<()> {
    let fixture = Fixture::current()?;
    let obsolete_images = fixture.study.join("containers/staging/obsolete");
    let obsolete_evidence = fixture.study.join("containers/evidence/image-campaigns/obsolete");
    fixture.add_admitted_image_to(
        b"superseded campaign image",
        &obsolete_images,
        &obsolete_evidence,
    )?;
    let request = ImageInventoryDerivationRequest {
        repository_root: fixture.repository.clone(),
        study_root: fixture.study.clone(),
        evidence_roots: vec![
            fixture.request.evidence_roots[0].clone(),
            ImageInventoryEvidenceRoots {
                image_root: fixture.study.join("containers/apptainer"),
                build_evidence_root: obsolete_evidence,
            },
        ],
        run_evidence_root: fixture.repository.join("containers/runs"),
    };

    let evidence = derive_image_inventory_evidence(&request)?;
    let tool = evidence.publication.tools.first().context("publication omits example")?;

    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.candidates.len(), 1);
    assert_eq!(evidence.scan.images.len(), 1);
    assert_eq!(evidence.scan.build_evidence.len(), 1);
    Ok(())
}

#[test]
fn derives_one_current_canonical_image_from_exact_evidence() -> Result<()> {
    let fixture = Fixture::current()?;
    let evidence = derive_image_inventory_evidence(&fixture.request)?;
    let publication = evidence.publication;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.candidates.len(), 1);
    assert!(tool.candidates[0].admitted);
    assert!(tool.candidates[0].current_recipe);
    assert!(tool.candidates[0].path.starts_with("study://containers/apptainer/"));
    assert_eq!(
        publication.publication.raw_schema_version,
        "bijux.container.image_inventory_scan.v2"
    );
    assert_eq!(evidence.scan.evidence_roots.len(), 1);
    assert_eq!(evidence.scan.evidence_roots[0].image_root, "study://containers/apptainer");
    assert_eq!(
        publication.publication.raw_sha256,
        bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec_pretty(&evidence.scan)?)
    );
    bijux_dna_containers::evidence::validate_image_inventory_evidence(
        &evidence.scan,
        &publication,
    )?;
    Ok(())
}

#[test]
fn derives_single_roster_after_governed_activation_retires_the_build_source() -> Result<()> {
    let fixture = Fixture::current()?;
    let active_root = fixture.study.join("containers/current-images");
    fs::create_dir_all(&active_root)?;
    let source = fs::read_dir(fixture.study.join("containers/apptainer"))?
        .next()
        .context("fixture omits the built image")??
        .path();
    let active = active_root.join(source.file_name().context("built image has no filename")?);
    fs::rename(&source, &active)?;
    fs::remove_dir(fixture.study.join("containers/apptainer"))?;
    let request = ImageInventoryDerivationRequest {
        repository_root: fixture.repository.clone(),
        study_root: fixture.study.clone(),
        evidence_roots: vec![ImageInventoryEvidenceRoots {
            image_root: active_root,
            build_evidence_root: fixture.study.join("containers/evidence"),
        }],
        run_evidence_root: fixture.repository.join("containers/runs"),
    };

    let evidence = derive_image_inventory_evidence(&request)?;
    let tool = evidence.publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert!(tool.candidates[0].path.starts_with("study://containers/current-images/"));
    assert_eq!(evidence.scan.images.len(), 1);
    assert_eq!(evidence.scan.build_evidence.len(), 1);
    Ok(())
}

#[test]
fn derives_activated_roster_from_nested_campaign_evidence() -> Result<()> {
    let fixture = Fixture::current()?;
    let source = fs::read_dir(fixture.study.join("containers/apptainer"))?
        .next()
        .context("fixture omits the built image")??
        .path();
    let active_root = fixture.study.join("containers/current-images");
    fs::create_dir_all(&active_root)?;
    fs::hard_link(
        &source,
        active_root.join(source.file_name().context("built image has no filename")?),
    )?;
    let campaign_evidence = fixture.study.join("containers/evidence/selected-example");
    fs::create_dir_all(&campaign_evidence)?;
    let evidence_path = fs::read_dir(fixture.study.join("containers/evidence"))?
        .find_map(|entry| {
            entry.ok().and_then(|entry| {
                entry.file_type().ok().filter(std::fs::FileType::is_file).map(|_| entry.path())
            })
        })
        .context("fixture omits build evidence")?;
    fs::rename(
        &evidence_path,
        campaign_evidence.join(evidence_path.file_name().context("evidence has no filename")?),
    )?;
    let request = ImageInventoryDerivationRequest {
        repository_root: fixture.repository.clone(),
        study_root: fixture.study.clone(),
        evidence_roots: vec![ImageInventoryEvidenceRoots {
            image_root: active_root,
            build_evidence_root: fixture.study.join("containers/evidence"),
        }],
        run_evidence_root: fixture.repository.join("containers/runs"),
    };

    let evidence = derive_image_inventory_evidence(&request)?;
    let tool = evidence.publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert!(tool.candidates[0].path.starts_with("study://containers/current-images/"));
    assert_eq!(evidence.scan.images.len(), 1);
    assert_eq!(evidence.scan.build_evidence.len(), 1);
    Ok(())
}

#[test]
fn composite_roster_admits_a_retired_build_source_with_unique_digest() -> Result<()> {
    let fixture = Fixture::current()?;
    let active_root = fixture.study.join("containers/current-images");
    let additional_images = fixture.study.join("containers/staging/additional");
    let additional_evidence = fixture.study.join("containers/evidence/additional");
    fs::create_dir_all(&active_root)?;
    fs::create_dir_all(&additional_images)?;
    fs::create_dir_all(&additional_evidence)?;
    let source = fs::read_dir(fixture.study.join("containers/apptainer"))?
        .next()
        .context("fixture omits the built image")??
        .path();
    let active = active_root.join(source.file_name().context("built image has no filename")?);
    fs::rename(&source, &active)?;
    fs::remove_dir(fixture.study.join("containers/apptainer"))?;
    let request = ImageInventoryDerivationRequest {
        repository_root: fixture.repository.clone(),
        study_root: fixture.study.clone(),
        evidence_roots: vec![
            ImageInventoryEvidenceRoots {
                image_root: active_root,
                build_evidence_root: fixture.study.join("containers/evidence"),
            },
            ImageInventoryEvidenceRoots {
                image_root: additional_images,
                build_evidence_root: additional_evidence,
            },
        ],
        run_evidence_root: fixture.repository.join("containers/runs"),
    };

    let evidence = derive_image_inventory_evidence(&request)?;
    let tool = evidence.publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(evidence.scan.evidence_roots.len(), 2);
    Ok(())
}

#[test]
fn retired_build_source_must_remain_under_the_governed_study_root() -> Result<()> {
    let fixture = Fixture::current()?;
    let evidence_path = fs::read_dir(fixture.study.join("containers/evidence"))?
        .next()
        .context("fixture omits build evidence")??
        .path();
    let mut evidence: serde_json::Value = serde_json::from_slice(&fs::read(&evidence_path)?)?;
    evidence["image"]["path"] = json!(fixture
        .repository
        .parent()
        .context("fixture repository has no parent")?
        .join("retired/image.sif"));
    fs::write(&evidence_path, serde_json::to_vec_pretty(&evidence)?)?;
    fs::remove_file(
        fs::read_dir(fixture.study.join("containers/apptainer"))?
            .next()
            .context("fixture omits the built image")??
            .path(),
    )?;

    let error = derive_image_inventory_evidence(&fixture.request)
        .err()
        .context("retired external build source was admitted")?;
    assert!(format!("{error:#}").contains("retired build-evidence image path is outside"));
    Ok(())
}

#[test]
fn run_reference_closure_is_scoped_to_scanned_images() -> Result<()> {
    let fixture = Fixture::current()?;
    fixture.reference_image_from_run(&"d".repeat(64))?;

    let evidence = derive_image_inventory_evidence(&fixture.request)?;

    assert!(evidence.scan.run_references.is_empty());
    assert!(evidence.publication.tools[0].candidates[0].run_references.is_empty());
    bijux_dna_containers::evidence::validate_image_inventory_evidence(
        &evidence.scan,
        &evidence.publication,
    )?;
    Ok(())
}

#[test]
fn historical_v1_scan_without_root_identities_remains_valid() -> Result<()> {
    let fixture = Fixture::current()?;
    let mut scan = derive_image_inventory_evidence(&fixture.request)?.scan;
    scan.schema_version = "bijux.container.image_inventory_scan.v1".to_string();
    scan.evidence_roots.clear();

    bijux_dna_containers::evidence::validate_image_inventory_scan(&scan)?;
    Ok(())
}

#[test]
fn current_scan_requires_replayable_root_identities() -> Result<()> {
    let fixture = Fixture::current()?;
    let mut scan = derive_image_inventory_evidence(&fixture.request)?.scan;
    scan.evidence_roots.clear();

    let error = bijux_dna_containers::evidence::validate_image_inventory_scan(&scan)
        .err()
        .map(|error| error.to_string());

    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("requires ordered evidence-root identities")));
    Ok(())
}

#[test]
fn admits_complete_flattened_build_evidence() -> Result<()> {
    let fixture = Fixture::current()?;
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "remove repository build context"])?;
    let digest = fixture.add_admitted_image(b"flattened-evidence image")?;
    fixture.rewrite_build_evidence_as_flattened(&digest)?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(digest.as_str()));
    Ok(())
}

#[test]
fn rejects_mixed_build_evidence_identity_encodings() -> Result<()> {
    let fixture = Fixture::current()?;
    let evidence_path = fs::read_dir(fixture.study.join("containers/evidence"))?
        .next()
        .context("fixture omits build evidence")??
        .path();
    let mut evidence: serde_json::Value = serde_json::from_slice(&fs::read(&evidence_path)?)?;
    evidence["definition_sha256"] = evidence["definition"]["sha256"].clone();
    fs::write(evidence_path, serde_json::to_vec_pretty(&evidence)?)?;

    let error = derive_image_inventory_publication(&fixture.request)
        .err()
        .context("mixed identity encodings were admitted")?;
    assert!(format!("{error:#}").contains("exactly one complete nested or flattened identity"));
    Ok(())
}

#[test]
fn classifies_every_unmatched_active_image() -> Result<()> {
    let fixture = Fixture::current()?;
    let bytes = b"unclassified image";
    let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
    fs::write(
        fixture
            .study
            .join("containers/apptainer")
            .join(format!("rogue-linux-amd64-{digest:.12}.sif")),
        bytes,
    )?;

    let evidence = derive_image_inventory_evidence(&fixture.request)?;
    assert_eq!(evidence.publication.unknown_images.len(), 1);
    assert_eq!(evidence.publication.unknown_images[0].sha256, digest);
    assert_eq!(
        evidence.publication.unknown_images[0].disposition,
        DetachedImageDisposition::Unclassified
    );
    assert_eq!(evidence.publication.summary.unknown_image_count, 1);
    bijux_dna_containers::evidence::validate_image_inventory_evidence(
        &evidence.scan,
        &evidence.publication,
    )?;
    Ok(())
}

#[test]
fn rejects_publication_that_omits_a_scanned_image() -> Result<()> {
    let fixture = Fixture::current()?;
    let bytes = b"unclassified image";
    let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
    fs::write(
        fixture
            .study
            .join("containers/apptainer")
            .join(format!("rogue-linux-amd64-{digest:.12}.sif")),
        bytes,
    )?;
    let mut evidence = derive_image_inventory_evidence(&fixture.request)?;
    evidence.publication.unknown_images.clear();
    evidence.publication.summary.unknown_image_count = 0;

    let error = bijux_dna_containers::evidence::validate_image_inventory_evidence(
        &evidence.scan,
        &evidence.publication,
    )
    .err()
    .context("publication omitted a scanned image")?;
    assert!(error.to_string().contains("classify every scanned active image"));
    Ok(())
}

#[test]
fn retains_referenced_historical_version_outside_active_catalog_prefix() -> Result<()> {
    let fixture = Fixture::current()?;
    let bytes = b"historical version image";
    let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
    fs::write(
        fixture
            .study
            .join("containers/apptainer")
            .join(format!("example-0.9.0-linux-amd64-{digest:.12}.sif")),
        bytes,
    )?;
    fixture.reference_image_from_run(&digest)?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    assert_eq!(publication.historical_images.len(), 1);
    let historical = &publication.historical_images[0];
    assert_eq!(historical.catalog_tool.as_deref(), Some("example"));
    assert_eq!(historical.real_run_references, 1);
    assert_eq!(historical.run_references.len(), 1);
    assert_eq!(publication.summary.historical_dependency_count, 1);
    Ok(())
}

#[test]
fn rejects_publication_that_omits_a_run_reference_identity() -> Result<()> {
    let fixture = Fixture::current()?;
    let bytes = b"historical version image";
    let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
    fs::write(
        fixture
            .study
            .join("containers/apptainer")
            .join(format!("example-0.9.0-linux-amd64-{digest:.12}.sif")),
        bytes,
    )?;
    fixture.reference_image_from_run(&digest)?;
    let mut evidence = derive_image_inventory_evidence(&fixture.request)?;
    evidence.publication.historical_images[0].run_references.clear();

    let error = bijux_dna_containers::evidence::validate_image_inventory_evidence(
        &evidence.scan,
        &evidence.publication,
    )
    .err()
    .context("publication omitted a run-reference identity")?;
    assert!(error.to_string().contains("exact run-reference closure"));
    Ok(())
}

#[test]
fn exposes_unreferenced_obsolete_catalog_candidate() -> Result<()> {
    let fixture = Fixture::current()?;
    let obsolete_digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "remove repository build context"])?;
    fixture.add_admitted_image(b"replacement image")?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert!(tool.candidates.iter().any(|candidate| {
        candidate.sha256 == obsolete_digest
            && candidate.disposition == ImageCandidateDisposition::UnreferencedObsolete
    }));
    assert_eq!(publication.summary.unreferenced_obsolete_count, 1);
    Ok(())
}

#[test]
fn records_interrupted_staged_images_separately() -> Result<()> {
    let fixture = Fixture::current()?;
    let staged_root = fixture.study.join("containers/apptainer/.building");
    fs::create_dir_all(&staged_root)?;
    fs::write(staged_root.join("example-staged.sif"), b"staged image")?;

    let evidence = derive_image_inventory_evidence(&fixture.request)?;
    assert_eq!(evidence.scan.staged_images.len(), 1);
    assert_eq!(evidence.publication.staged_images.len(), 1);
    assert_eq!(evidence.publication.summary.staged_image_count, 1);
    Ok(())
}

#[test]
fn preserves_one_admitted_image_as_historical_after_recipe_change() -> Result<()> {
    let fixture = Fixture::current()?;
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n\n%post\n  true\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "change example recipe"])?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Historical);
    assert!(!tool.candidates[0].current_recipe);
    Ok(())
}

#[test]
fn selects_the_unique_descendant_when_multiple_images_become_historical() -> Result<()> {
    let fixture = Fixture::current()?;
    let prior_digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fs::write(fixture.repository.join("README.md"), "new source authority\n")?;
    git(&fixture.repository, &["add", "README.md"])?;
    git(&fixture.repository, &["commit", "-qm", "advance source authority"])?;
    let descendant_digest = fixture.add_admitted_image(b"descendant current rebuild")?;
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n\n%post\n  true\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "change example recipe"])?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Historical);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(descendant_digest.as_str()));
    assert!(tool.candidates.iter().any(|candidate| {
        candidate.sha256 == prior_digest
            && candidate.disposition == ImageCandidateDisposition::UnreferencedObsolete
    }));
    bijux_dna_containers::evidence::validate_image_inventory_publication(&publication)?;
    Ok(())
}

#[test]
fn current_recipe_state_requires_the_same_evidence_to_be_admissible() -> Result<()> {
    let fixture = Fixture::current()?;
    let digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n\n%post\n  true\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "change example recipe"])?;

    let current_commit = repository_head_commit(&fixture.repository)?;
    let current_recipe = apptainer_recipe_identity_at_commit(
        &fixture.repository,
        &current_commit,
        "containers/apptainer/example.def",
    )?;
    let evidence_path =
        fixture.study.join("containers/evidence").join(format!("{digest}.build.json"));
    let mut forged_current: serde_json::Value = serde_json::from_slice(&fs::read(&evidence_path)?)?;
    forged_current["definition"]["sha256"] = json!(current_recipe.definition_sha256);
    forged_current["build_context"] = json!(current_recipe.build_context);
    fs::write(
        fixture.study.join("containers/evidence/forged-current.build.json"),
        serde_json::to_vec_pretty(&forged_current)?,
    )?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Historical);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(digest.as_str()));
    assert!(!tool.candidates[0].current_recipe);
    Ok(())
}

#[test]
fn retains_repository_run_dependency_beside_new_current_image() -> Result<()> {
    let fixture = Fixture::current()?;
    let historical_digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fixture.reference_image_from_run(&historical_digest)?;
    fs::write(
        fixture.repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%files\n  containers/tools/example/example /usr/local/bin/example\n\n%post\n  true\n",
    )?;
    git(&fixture.repository, &["add", "containers/apptainer/example.def"])?;
    git(&fixture.repository, &["commit", "-qm", "change example recipe"])?;
    let current_digest = fixture.add_admitted_image(b"image from the current recipe")?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(current_digest.as_str()));
    assert_eq!(tool.candidates.len(), 2);
    assert!(tool.candidates.iter().any(|candidate| {
        candidate.sha256 == historical_digest
            && matches!(
                candidate.disposition,
                bijux_dna_containers::evidence::ImageCandidateDisposition::HistoricalRunDependency
            )
    }));
    Ok(())
}

#[test]
fn selects_unique_unreferenced_rebuild_beside_current_run_dependency() -> Result<()> {
    let fixture = Fixture::current()?;
    let historical_digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fixture.reference_image_from_run(&historical_digest)?;
    let rebuild_digest = fixture.add_admitted_image(b"unreferenced current rebuild")?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(rebuild_digest.as_str()));
    assert_eq!(tool.candidates.len(), 2);
    assert!(tool.candidates.iter().any(|candidate| {
        candidate.sha256 == historical_digest
            && matches!(
                candidate.disposition,
                bijux_dna_containers::evidence::ImageCandidateDisposition::HistoricalRunDependency
            )
    }));
    Ok(())
}

#[test]
fn selects_unique_descendant_build_beside_unreferenced_current_image() -> Result<()> {
    let fixture = Fixture::current()?;
    let prior_digest = bijux_dna_infra::hash_bytes_sha256(b"current immutable image");
    fs::write(fixture.repository.join("README.md"), "new source authority\n")?;
    git(&fixture.repository, &["add", "README.md"])?;
    git(&fixture.repository, &["commit", "-qm", "advance source authority"])?;
    let descendant_digest = fixture.add_admitted_image(b"descendant current rebuild")?;

    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Current);
    assert_eq!(tool.selected_image_sha256.as_deref(), Some(descendant_digest.as_str()));
    assert!(tool.candidates.iter().any(|candidate| {
        candidate.sha256 == prior_digest
            && candidate.disposition == ImageCandidateDisposition::UnreferencedObsolete
    }));
    Ok(())
}

#[test]
fn publishes_ambiguous_current_images_without_selecting_either() -> Result<()> {
    let fixture = Fixture::current()?;
    fixture.add_admitted_image(b"another image from the same recipe")?;
    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Ambiguous);
    assert!(tool.selected_image_sha256.is_none());
    assert_eq!(tool.candidates.iter().filter(|candidate| candidate.admitted).count(), 2);
    bijux_dna_containers::evidence::validate_image_inventory_publication(&publication)?;
    Ok(())
}

#[test]
fn publishes_missing_when_recorded_image_size_does_not_match_storage() -> Result<()> {
    let fixture = Fixture::current()?;
    let evidence_path = fs::read_dir(fixture.study.join("containers/evidence"))?
        .next()
        .context("fixture omits build evidence")??
        .path();
    let mut evidence: serde_json::Value = serde_json::from_slice(&fs::read(&evidence_path)?)?;
    evidence["image"]["size_bytes"] = json!(1);
    fs::write(evidence_path, serde_json::to_vec_pretty(&evidence)?)?;
    let publication = derive_image_inventory_publication(&fixture.request)?;
    let tool = publication.tools.first().context("publication omits example")?;
    assert_eq!(tool.state, ContainerImageState::Missing);
    assert!(tool.selected_image_sha256.is_none());
    assert!(tool.candidates.iter().all(|candidate| !candidate.admitted));
    Ok(())
}

fn git(repository: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git").arg("-C").arg(repository).args(args).output()?;
    if !output.status.success() {
        anyhow::bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn catalog_toml() -> &'static str {
    r#"schema_version = "bijux.container.catalog.v1"
owner = "Bijux"
contact = "test@bijux.io"
repository = "https://example.invalid/bijux-genomics"
apptainer_architecture = "linux-amd64"
docker_architecture = "linux-arm64"
evidence_schema = "bijux.container.build_evidence.v1"
recipe_license = "MIT"
required_checks = ["version"]
evidence_dimensions = ["identity"]

[[tools]]
id = "example"
display_name = "Example"
purpose = "Fixture"
version = "1.0.0"
product_support = "supported"
artifact_delivery = "redistributable_image"
redistribution_permitted = true
upstream = "https://example.invalid"
documentation = "https://example.invalid/docs"
license = "MIT"
domains = ["fixture"]
stage_ids = ["fixture"]
expected_bin = "example"
version_command = "example --version"
help_command = "example --help"
expected_version_regex = "1[.]0[.]0"
apptainer_definition = "containers/apptainer/example.def"
apptainer_supported = true
dockerfile = "containers/docker/example.Dockerfile"
docker_arm64_supported = false
docker_arm64_reason = "fixture"
build_network = false
runtime_network = false
real_test_stage = "fixture"
real_test_input_kind = "fixture"
"#
}
