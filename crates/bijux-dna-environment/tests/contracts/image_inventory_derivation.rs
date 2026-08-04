use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use bijux_dna_environment::evidence::{
    apptainer_recipe_identity_at_commit, derive_image_inventory_evidence,
    derive_image_inventory_publication, repository_head_commit, ContainerImageState,
    ImageInventoryDerivationRequest,
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
            image_root: study.join("containers/apptainer"),
            build_evidence_root: study.join("containers/evidence"),
            run_evidence_root: repository.join("containers/runs"),
        };
        let fixture = Self { _temporary: temporary, repository, study, request };
        fixture.add_admitted_image(b"current immutable image")?;
        Ok(fixture)
    }

    fn add_admitted_image(&self, bytes: &[u8]) -> Result<String> {
        let digest = bijux_dna_infra::hash_bytes_sha256(bytes);
        let filename = format!("example-1.0.0-linux-amd64-{digest:.12}.sif");
        let image = self.study.join("containers/apptainer").join(&filename);
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
            self.study.join("containers/evidence").join(format!("{digest}.build.json")),
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
        "bijux.container.image_inventory_scan.v1"
    );
    assert_eq!(
        publication.publication.raw_sha256,
        bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec_pretty(&evidence.scan)?)
    );
    bijux_dna_environment::evidence::validate_image_inventory_evidence(
        &evidence.scan,
        &publication,
    )?;
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
                bijux_dna_environment::evidence::ImageCandidateDisposition::HistoricalRunDependency
            )
    }));
    Ok(())
}

#[test]
fn rejects_multiple_images_admitted_for_the_current_recipe() -> Result<()> {
    let fixture = Fixture::current()?;
    fixture.add_admitted_image(b"another image from the same recipe")?;
    let error = derive_image_inventory_publication(&fixture.request)
        .err()
        .context("ambiguous current images were admitted")?;
    assert!(error.to_string().contains("2 admitted images for the current recipe"));
    Ok(())
}

#[test]
fn rejects_image_whose_recorded_size_does_not_match_storage() -> Result<()> {
    let fixture = Fixture::current()?;
    let evidence_path = fs::read_dir(fixture.study.join("containers/evidence"))?
        .next()
        .context("fixture omits build evidence")??
        .path();
    let mut evidence: serde_json::Value = serde_json::from_slice(&fs::read(&evidence_path)?)?;
    evidence["image"]["size_bytes"] = json!(1);
    fs::write(evidence_path, serde_json::to_vec_pretty(&evidence)?)?;
    let error = derive_image_inventory_publication(&fixture.request)
        .err()
        .context("tampered image metadata was admitted")?;
    assert!(error.to_string().contains("has no admitted immutable image"));
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
support_disposition = "retained"
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
