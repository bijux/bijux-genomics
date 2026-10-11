#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use bijux_dna_containers::evidence::tool_contract::{
    derive_apptainer_tool_contract, tool_contract_labels, verify_apptainer_tool_contract,
};
use bijux_dna_containers::evidence::{
    apptainer_recipe_identity_at_commit, derive_image_inventory_evidence,
    execute_runtime_verification, repository_head_commit, verify_runtime_verification_report,
    ImageInventoryDerivationRequest, RuntimeEvidenceStatus, RuntimeVerificationRequest,
    RuntimeVerificationResult,
};
use serde_json::json;

#[test]
fn executes_canonical_inventory_and_binds_immutable_manifests() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let repository = temporary.path().join("repository");
    let study = temporary.path().join("study");
    fs::create_dir_all(repository.join("containers/contracts"))?;
    fs::create_dir_all(repository.join("containers/apptainer"))?;
    fs::create_dir_all(repository.join("containers/tools/example"))?;
    fs::create_dir_all(repository.join("containers/runs"))?;
    fs::create_dir_all(study.join("containers/apptainer"))?;
    fs::create_dir_all(study.join("containers/evidence"))?;
    fs::create_dir_all(study.join("data"))?;
    fs::write(repository.join("containers/contracts/catalog.toml"), catalog())?;
    fs::write(
        repository.join("containers/apptainer/example.def"),
        "Bootstrap: docker\nFrom: scratch\n\n%labels\n    org.opencontainers.image.title example\n\n%files\n    containers/tools/example/example /usr/local/bin/example\n",
    )?;
    fs::write(repository.join("containers/tools/example/example"), "#!/bin/sh\n")?;
    git(&repository, &["init", "-q"])?;
    git(&repository, &["config", "user.name", "Bijux Test"])?;
    git(&repository, &["config", "user.email", "test@bijux.io"])?;
    git(&repository, &["add", "containers"])?;
    git(&repository, &["commit", "-qm", "add runtime fixture"])?;

    let image_bytes = b"immutable fixture image";
    let image_sha256 = bijux_dna_infra::hash_bytes_sha256(image_bytes);
    let image_path = study
        .join("containers/apptainer")
        .join(format!("example-1.0.0-linux-amd64-{image_sha256:.12}.sif"));
    fs::write(&image_path, image_bytes)?;
    let commit = repository_head_commit(&repository)?;
    let recipe = apptainer_recipe_identity_at_commit(
        &repository,
        &commit,
        "containers/apptainer/example.def",
    )?;
    fs::write(
        study.join("containers/evidence/example.build.json"),
        serde_json::to_vec_pretty(&json!({
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
                "path": image_path,
                "sha256": image_sha256,
                "size_bytes": image_bytes.len(),
            },
        }))?,
    )?;
    let inventory_request = ImageInventoryDerivationRequest {
        repository_root: repository.clone(),
        study_root: study.clone(),
        image_root: study.join("containers/apptainer"),
        build_evidence_root: study.join("containers/evidence"),
        run_evidence_root: repository.join("containers/runs"),
    };
    let inventory = derive_image_inventory_evidence(&inventory_request)?.publication;
    let inventory_path = study.join("containers/evidence/inventory.json");
    bijux_dna_infra::atomic_create_json(&inventory_path, &inventory)?;

    let contract = derive_apptainer_tool_contract(&repository, "example", &commit, "linux-amd64")?;
    let apptainer = fake_apptainer(temporary.path(), &contract)?;
    let evidence_root = study.join("containers/evidence/runtime").join(&commit);
    let publication_path = evidence_root.join("publication/runtime-verification.json");
    let markdown_path = evidence_root.join("publication/runtime-verification.md");
    let request = RuntimeVerificationRequest {
        repository_root: repository,
        study_root: study,
        inventory_path,
        apptainer_executable: apptainer,
        data_root: inventory_request.study_root.join("data"),
        evidence_root: evidence_root.clone(),
        worker_count: 2,
        timeout: Duration::from_secs(2),
    };
    let result = execute_runtime_verification(&request, &publication_path, &markdown_path)?;

    assert_runtime_evidence(&result, &evidence_root)?;
    verify_runtime_verification_report(&publication_path, &markdown_path)?;

    assert_label_tampering_rejected(
        temporary.path(),
        &contract,
        &request,
        &image_path,
        &image_sha256,
    )?;
    Ok(())
}

fn assert_runtime_evidence(result: &RuntimeVerificationResult, evidence_root: &Path) -> Result<()> {
    assert_eq!((result.passed, result.failed), (1, 0));
    let tool = result.publication.tools.get("example").context("missing runtime tool")?;
    assert_eq!(tool.status, RuntimeEvidenceStatus::Passed);
    let prefix = "study://containers/evidence/runtime/";
    assert!(tool.identity.as_ref().is_some_and(|identity| identity.path.starts_with(prefix)));
    assert!(tool.manifest.as_ref().is_some_and(|manifest| manifest.path.starts_with(prefix)));
    assert!(evidence_root.join("manifests/example.json").is_file());
    assert!(evidence_root.join("identities/example.json").is_file());
    assert_eq!(fs::read_dir(evidence_root.join("logs/example"))?.count(), 4);
    Ok(())
}

fn assert_label_tampering_rejected(
    root: &Path,
    contract: &bijux_dna_containers::evidence::tool_contract::ContainerToolContract,
    request: &RuntimeVerificationRequest,
    image_path: &Path,
    image_sha256: &str,
) -> Result<()> {
    let mut labels = tool_contract_labels(contract);
    labels.insert("io.bijux.tool-contract.version".to_string(), "stale".to_string());
    fs::write(
        root.join("labels.json"),
        serde_json::to_vec_pretty(&json!({"data": {"attributes": {"labels": labels}}}))?,
    )?;
    let Err(error) = verify_apptainer_tool_contract(
        &request.apptainer_executable,
        image_path,
        image_sha256,
        contract,
    ) else {
        bail!("changed labels passed direct image identity verification");
    };
    let detail = format!("{error:#}");
    assert!(detail.contains("mismatches embedded contract"), "{detail}");
    let publication_path = request.evidence_root.join("publication/replayed-verification.json");
    let markdown_path = request.evidence_root.join("publication/replayed-verification.md");
    let Err(error) = execute_runtime_verification(request, &publication_path, &markdown_path)
    else {
        bail!("changed image labels did not invalidate replayed runtime evidence");
    };
    assert!(error.to_string().contains("runtime verification execution errors"));
    Ok(())
}

fn fake_apptainer(
    root: &Path,
    contract: &bijux_dna_containers::evidence::tool_contract::ContainerToolContract,
) -> Result<std::path::PathBuf> {
    let contract_path = root.join("tool-contract.json");
    bijux_dna_infra::atomic_create_json(&contract_path, contract)?;
    let labels_path = root.join("labels.json");
    bijux_dna_infra::atomic_create_json(
        &labels_path,
        &json!({"data": {"attributes": {"labels": tool_contract_labels(contract)}}}),
    )?;
    let executable = root.join("apptainer");
    fs::write(
        &executable,
        format!(
            "#!/bin/sh\ncase \"$1\" in\n  inspect) cat '{}' ;;\n  exec)\n    case \"$*\" in\n      *tool-contract.json*) cat '{}' ;;\n      *--version*) echo 'example 1.0.0' ;;\n      *--help*) echo 'example help' ;;\n      *) exit 2 ;;\n    esac\n    ;;\n  *) exit 2 ;;\nesac\n",
            labels_path.display(),
            contract_path.display(),
        ),
    )?;
    let mut permissions = fs::metadata(&executable)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions)?;
    Ok(executable)
}

fn git(repository: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git").arg("-C").arg(repository).args(args).output()?;
    if !output.status.success() {
        anyhow::bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn catalog() -> &'static str {
    r#"schema_version = "bijux.container.catalog.v1"
owner = "Bijux"
contact = "test@bijux.io"
repository = "https://example.invalid/bijux-genomics"
apptainer_architecture = "linux-amd64"
docker_architecture = "linux-arm64"
evidence_schema = "bijux.container.build_evidence.v1"
recipe_license = "MIT"
required_checks = ["version", "help"]
evidence_dimensions = ["identity", "runtime"]

[[tools]]
id = "example"
display_name = "Example"
purpose = "Fixture"
version = "1.0.0"
support_disposition = "retained"
upstream = "https://example.invalid"
documentation = "https://example.invalid/docs"
license = "MIT"
source_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
pinned_commit = ""
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
