#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    clean_repository_head_commit, derive_image_inventory_evidence, prepare_apptainer_tool_contract,
    publish_canonical_apptainer_image, tool_contract_labels,
    verify_apptainer_image_campaign_receipt, CanonicalApptainerImageDestination,
    CanonicalApptainerImagePublicationRequest, ContainerImageState, ImageCampaignJobCompletion,
    ImageInventoryDerivationRequest, ToolContractBuildRequest,
    IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1,
};

#[test]
fn canonical_publication_closes_build_inspection_and_inventory_identity() -> Result<()> {
    let repository = tempfile::tempdir()?;
    let study = tempfile::tempdir()?;
    write_repository_fixture(repository.path())?;
    commit_fixture(repository.path())?;
    let commit = clean_repository_head_commit(repository.path())?;

    let preview_root = repository.path().join("artifacts/containers/tool-contract-build-inputs");
    let prepared = prepare_apptainer_tool_contract(&ToolContractBuildRequest {
        repository_root: repository.path().to_path_buf(),
        tool_id: "contract_tool".to_string(),
        repository_commit: commit.clone(),
        architecture: "linux/amd64".to_string(),
        output_root: preview_root.clone(),
    })?;
    let labels = tool_contract_labels(&prepared.contract).into_iter().collect::<BTreeMap<_, _>>();
    let labels_path = study.path().join("labels.json");
    bijux_dna_infra::atomic_write_json(
        &labels_path,
        &serde_json::json!({"data": {"attributes": {"labels": labels}}}),
    )?;
    fs::remove_dir_all(repository.path().join("artifacts"))?;

    let executable = study.path().join("apptainer");
    let contract_path = preview_root.join("contract_tool/tool-contract.json");
    write_executable(
        &executable,
        &format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo 'apptainer version 1.4.2' ;;\n  build) printf 'canonical-sif' > \"$3\" ;;\n  exec) cat {} ;;\n  inspect) cat {} ;;\n  *) exit 64 ;;\nesac\n",
            shell_literal(&contract_path),
            shell_literal(&labels_path),
        ),
    )?;
    let publication =
        publish_canonical_apptainer_image(&CanonicalApptainerImagePublicationRequest {
            repository_root: repository.path().to_path_buf(),
            study_root: study.path().to_path_buf(),
            tool_id: "contract_tool".to_string(),
            architecture: "linux/amd64".to_string(),
            apptainer_executable: executable,
            maximum_build_invocations: 1,
            retry_delay: Duration::ZERO,
            lock_timeout: Duration::from_secs(1),
            destination: CanonicalApptainerImageDestination::CommitCampaign,
        })?;

    assert_eq!(publication.status, "passed");
    assert_eq!(publication.image.sha256.len(), 64);
    assert_eq!(publication.base_image_digest, "b".repeat(64));
    assert_eq!(publication.apptainer_version, "apptainer version 1.4.2");
    assert!(publication.finished_at_unix_ms >= publication.started_at_unix_ms);
    let campaign_image_prefix = format!("study://containers/staging/{commit}/");
    let campaign_evidence_prefix = format!("study://containers/evidence/image-campaigns/{commit}/");
    assert!(publication.image.path.starts_with(&campaign_image_prefix));
    assert!(publication.build_log.path.starts_with(&campaign_evidence_prefix));
    assert!(publication.evidence_path.starts_with(&campaign_evidence_prefix));
    assert!(publication.identity.path.starts_with(&campaign_evidence_prefix));
    assert!(publication.evidence_path.ends_with(".build.json"));
    assert!(study_path(study.path(), &publication.image.path)?.is_file());
    assert!(study_path(study.path(), &publication.evidence_path)?.is_file());
    assert_eq!(
        fs::read_dir(study.path().join(format!("containers/staging/{commit}/.building")))?.count(),
        0
    );

    let run_root = study.path().join("containers/runs");
    fs::create_dir_all(&run_root)?;
    let inventory = derive_image_inventory_evidence(&ImageInventoryDerivationRequest {
        repository_root: repository.path().to_path_buf(),
        study_root: study.path().to_path_buf(),
        evidence_roots: vec![bijux_dna_containers::evidence::ImageInventoryEvidenceRoots {
            image_root: study.path().join(format!("containers/staging/{commit}")),
            build_evidence_root: study
                .path()
                .join(format!("containers/evidence/image-campaigns/{commit}")),
        }],
        run_evidence_root: run_root,
    })?;
    assert_eq!(inventory.publication.tools.len(), 1);
    assert_eq!(inventory.publication.tools[0].state, ContainerImageState::Current);
    assert_eq!(
        inventory.publication.tools[0].selected_image_sha256.as_deref(),
        Some(publication.image.sha256.as_str())
    );
    Ok(())
}

#[test]
fn clean_commit_identity_rejects_uncommitted_build_inputs() -> Result<()> {
    let repository = tempfile::tempdir()?;
    write_repository_fixture(repository.path())?;
    commit_fixture(repository.path())?;
    assert_eq!(clean_repository_head_commit(repository.path())?.len(), 40);
    fs::write(repository.path().join("containers/apptainer/contract_tool.def"), "changed")?;
    assert!(clean_repository_head_commit(repository.path())
        .is_err_and(|error| error.to_string().contains("repository changes")));
    Ok(())
}

#[test]
fn campaign_receipt_rejects_execution_digest_mutation() -> Result<()> {
    let attempt = tempfile::tempdir()?;
    let execution_path = attempt.path().join("campaign-execution.json");
    fs::write(&execution_path, b"{}\n")?;
    let completion = ImageCampaignJobCompletion {
        schema_version: IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1.to_string(),
        attempt_id: "fixture-attempt-001".to_string(),
        repository_commit: "a".repeat(40),
        slurm_job_id: "12345".to_string(),
        completed_at: "2026-08-08T19:45:53Z".to_string(),
        campaign_execution: "campaign-execution.json".to_string(),
        campaign_execution_sha256: "b".repeat(64),
    };
    bijux_dna_infra::atomic_write_json(&attempt.path().join("completion.json"), &completion)?;
    assert!(verify_apptainer_image_campaign_receipt(
        Path::new("/unreached/repository"),
        Path::new("/unreached/study"),
        Path::new("/unreached/apptainer"),
        attempt.path(),
    )
    .is_err_and(|error| error.to_string().contains("digest differs")));
    Ok(())
}

fn write_repository_fixture(root: &Path) -> Result<()> {
    fs::write(root.join(".gitignore"), "/artifacts/\n")?;
    fs::create_dir_all(root.join("containers/contracts"))?;
    fs::create_dir_all(root.join("containers/apptainer"))?;
    fs::write(
        root.join("containers/contracts/catalog.toml"),
        r#"schema_version = "bijux.container.catalog.v1"
owner = "fixture"
contact = "fixture@example.invalid"
repository = "fixture"
apptainer_architecture = "linux/amd64"
docker_architecture = "linux/arm64"
evidence_schema = "bijux.container.build_evidence.v1"
recipe_license = "MIT"
required_checks = []
evidence_dimensions = []

[[tools]]
id = "contract_tool"
display_name = "Contract Tool"
purpose = "contract fixture"
version = "1.0"
support_disposition = "retained"
support_reason = ""
upstream = "https://example.invalid"
documentation = "https://example.invalid/docs"
license = "MIT"
source_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
pinned_commit = ""
domains = ["fixture"]
stage_ids = ["fixture.stage"]
expected_bin = "contract-tool"
version_command = "contract-tool --version"
help_command = "contract-tool --help"
expected_version_regex = "1\\.0"
apptainer_definition = "containers/apptainer/contract_tool.def"
apptainer_supported = true
dockerfile = "containers/docker/contract_tool.Dockerfile"
docker_arm64_supported = false
docker_arm64_reason = "fixture"
build_network = false
runtime_network = false
real_test_stage = "fixture.stage"
real_test_input_kind = "fixture"
"#,
    )?;
    fs::write(
        root.join("containers/apptainer/contract_tool.def"),
        format!(
            "Bootstrap: docker\nFrom: ubuntu@sha256:{}\n\n%labels\n    org.opencontainers.image.title contract-tool\n\n%post\n    true\n",
            "b".repeat(64)
        ),
    )?;
    Ok(())
}

fn commit_fixture(root: &Path) -> Result<()> {
    run_git(root, &["init", "--quiet"])?;
    run_git(root, &["config", "user.email", "fixture@example.invalid"])?;
    run_git(root, &["config", "user.name", "Fixture"])?;
    run_git(root, &["add", ".gitignore", "containers"])?;
    run_git(root, &["commit", "--quiet", "-m", "test: add container fixture"])
}

fn run_git(root: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git").arg("-C").arg(root).args(args).output()?;
    if !output.status.success() {
        bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn write_executable(path: &Path, body: &str) -> Result<()> {
    fs::write(path, body)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn shell_literal(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

fn study_path(root: &Path, identity: &str) -> Result<std::path::PathBuf> {
    let relative = identity.strip_prefix("study://").context("identity is not study-relative")?;
    Ok(root.join(relative))
}
