use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::evidence::tool_contract::{
    apptainer_recipe_identity_at_commit, clean_repository_head_commit,
    prepare_apptainer_tool_contract, verify_apptainer_tool_contract, PreparedToolContractBuild,
    ToolContractBuildRequest, ToolImageIdentityEvidence,
};
use crate::evidence::{BuildContextIdentity, EvidenceSourceIdentity};

use super::{build_apptainer_image, ApptainerBuildAttempt, ApptainerBuildRequest};

pub const CONTAINER_BUILD_EVIDENCE_SCHEMA_V1: &str = "bijux.container.build_evidence.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalApptainerImageDestination {
    Active,
    CommitCampaign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalApptainerImagePublicationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub tool_id: String,
    pub architecture: String,
    pub apptainer_executable: PathBuf,
    pub maximum_build_invocations: u32,
    pub retry_delay: Duration,
    pub lock_timeout: Duration,
    pub destination: CanonicalApptainerImageDestination,
}

impl CanonicalApptainerImagePublicationRequest {
    #[must_use]
    pub fn retained(repository_root: PathBuf, study_root: PathBuf, tool_id: String) -> Self {
        Self {
            repository_root,
            study_root,
            tool_id,
            architecture: "linux/amd64".to_string(),
            apptainer_executable: PathBuf::from("apptainer"),
            maximum_build_invocations: 3,
            retry_delay: Duration::from_secs(2),
            lock_timeout: Duration::from_secs(30),
            destination: CanonicalApptainerImageDestination::Active,
        }
    }

    #[must_use]
    pub fn retained_campaign(
        repository_root: PathBuf,
        study_root: PathBuf,
        tool_id: String,
    ) -> Self {
        Self {
            destination: CanonicalApptainerImageDestination::CommitCampaign,
            ..Self::retained(repository_root, study_root, tool_id)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDefinitionIdentity {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalImageIdentity {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalBuildLogIdentity {
    pub path: String,
    pub sha256: String,
    pub attempt: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalApptainerImagePublication {
    pub schema_version: String,
    pub tool: String,
    pub version: String,
    pub status: String,
    pub repository_commit: String,
    pub architecture: String,
    pub build_evidence_id: String,
    pub catalog_sha256: String,
    pub source_sha256: Option<String>,
    pub pinned_commit: Option<String>,
    pub interface_sha256: String,
    pub definition: CanonicalDefinitionIdentity,
    pub build_context: Vec<BuildContextIdentity>,
    pub base_image_reference: String,
    pub base_image_digest: String,
    pub builder_host: String,
    pub apptainer_version: String,
    pub build_command: Vec<String>,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: u64,
    pub image: CanonicalImageIdentity,
    pub identity: EvidenceSourceIdentity,
    pub build_log: CanonicalBuildLogIdentity,
    pub evidence_path: String,
}

#[must_use]
pub fn container_build_evidence_schema() -> Value {
    serde_json::to_value(schema_for!(CanonicalApptainerImagePublication))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the canonical container-build evidence schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_container_build_evidence_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &container_build_evidence_schema())?;
    Ok(())
}

struct CanonicalContainerRoots {
    image: PathBuf,
    evidence: PathBuf,
    staging: PathBuf,
    build_logs: PathBuf,
}

struct PublishedCanonicalImage {
    base_image_reference: String,
    base_image_digest: String,
    builder_host: String,
    apptainer_version: String,
    build_command: Vec<String>,
    final_path: PathBuf,
    image_sha256: String,
    image_size_bytes: u64,
    identity_evidence: ToolImageIdentityEvidence,
    final_attempt: ApptainerBuildAttempt,
}

/// Build, inspect, and publish one retained image into the governed study container roots.
///
/// # Errors
/// Returns an error when source state is dirty, the catalog identity is invalid, a build or
/// inspection fails, staging is occupied, or immutable image/evidence publication conflicts.
pub fn publish_canonical_apptainer_image(
    request: &CanonicalApptainerImagePublicationRequest,
) -> Result<CanonicalApptainerImagePublication> {
    validate_request(request)?;
    let repository_commit = clean_repository_head_commit(&request.repository_root)?;
    let roots =
        canonical_container_roots(&request.study_root, &repository_commit, request.destination)?;
    let build_input_root =
        request.repository_root.join("artifacts/containers/tool-contract-build-inputs");
    let lock_path = roots.evidence.join(".build-locks").join(format!("{}.lock", request.tool_id));
    let _lock = bijux_dna_infra::FileLock::acquire(&lock_path, request.lock_timeout)
        .with_context(|| format!("acquire canonical image lock for {}", request.tool_id))?;

    let prepared = prepare_apptainer_tool_contract(&ToolContractBuildRequest {
        repository_root: request.repository_root.clone(),
        tool_id: request.tool_id.clone(),
        repository_commit: repository_commit.clone(),
        architecture: request.architecture.clone(),
        output_root: build_input_root,
    })?;
    let recipe = apptainer_recipe_identity_at_commit(
        &request.repository_root,
        &repository_commit,
        &prepared.contract.recipe_path,
    )?;
    let built = build_verified_image(request, &roots, &prepared, &repository_commit)?;
    let build_log_sha256 = bijux_dna_infra::hash_file_sha256(&built.final_attempt.log_path)
        .with_context(|| format!("hash build log {}", built.final_attempt.log_path.display()))?;
    let identity_path = roots.evidence.join(format!(
        "{}-{}-{:.12}.identity.json",
        request.tool_id, prepared.contract.build_evidence_id, built.image_sha256
    ));
    publish_identity_noclobber(&identity_path, &built.identity_evidence)?;
    let identity_sha256 = bijux_dna_infra::hash_file_sha256(&identity_path)
        .with_context(|| format!("hash image identity evidence {}", identity_path.display()))?;
    let evidence_path = roots.evidence.join(format!(
        "{}-{}-{:.12}.build.json",
        request.tool_id, prepared.contract.build_evidence_id, built.image_sha256
    ));
    let publication = CanonicalApptainerImagePublication {
        schema_version: CONTAINER_BUILD_EVIDENCE_SCHEMA_V1.to_string(),
        tool: request.tool_id.clone(),
        version: prepared.contract.version,
        status: "passed".to_string(),
        repository_commit,
        architecture: request.architecture.clone(),
        build_evidence_id: prepared.contract.build_evidence_id,
        catalog_sha256: prepared.contract.catalog_sha256,
        source_sha256: prepared.contract.source_sha256,
        pinned_commit: prepared.contract.pinned_commit,
        interface_sha256: prepared.contract.interface_sha256,
        definition: CanonicalDefinitionIdentity {
            path: recipe.definition_path,
            sha256: recipe.definition_sha256,
        },
        build_context: recipe.build_context,
        base_image_reference: built.base_image_reference,
        base_image_digest: built.base_image_digest,
        builder_host: built.builder_host,
        apptainer_version: built.apptainer_version,
        build_command: built.build_command,
        started_at_unix_ms: built.final_attempt.started_at_unix_ms,
        finished_at_unix_ms: built.final_attempt.finished_at_unix_ms,
        image: CanonicalImageIdentity {
            path: portable_study_path(&built.final_path, &request.study_root)?,
            sha256: built.image_sha256,
            size_bytes: built.image_size_bytes,
        },
        identity: EvidenceSourceIdentity {
            path: portable_study_path(&identity_path, &request.study_root)?,
            sha256: identity_sha256,
        },
        build_log: CanonicalBuildLogIdentity {
            path: portable_study_path(&built.final_attempt.log_path, &request.study_root)?,
            sha256: build_log_sha256,
            attempt: built.final_attempt.attempt,
        },
        evidence_path: portable_study_path(&evidence_path, &request.study_root)?,
    };
    publish_evidence_noclobber(&evidence_path, &publication)?;
    Ok(publication)
}

fn build_verified_image(
    request: &CanonicalApptainerImagePublicationRequest,
    roots: &CanonicalContainerRoots,
    prepared: &PreparedToolContractBuild,
    repository_commit: &str,
) -> Result<PublishedCanonicalImage> {
    let staging_path = roots
        .staging
        .join(format!("{}-{}.sif", request.tool_id, prepared.contract.build_evidence_id));
    if staging_path.exists() {
        bail!(
            "canonical image staging path already exists and requires reconciliation: {}",
            staging_path.display()
        );
    }
    let (base_image_reference, base_image_digest) =
        pinned_base_image_identity(&prepared.rendered_definition_path)?;
    let builder_host = command_text(Path::new("hostname"), &[], "builder hostname")?;
    let apptainer_version =
        command_text(&request.apptainer_executable, &["--version"], "Apptainer version")?;
    let build_command = vec![
        request.apptainer_executable.display().to_string(),
        "build".to_string(),
        "--force".to_string(),
        staging_path.display().to_string(),
        prepared.rendered_definition_path.display().to_string(),
    ];
    let build = build_apptainer_image(&ApptainerBuildRequest {
        executable: request.apptainer_executable.clone(),
        definition_path: prepared.rendered_definition_path.clone(),
        image_path: staging_path.clone(),
        evidence_root: roots.build_logs.clone(),
        tool_id: request.tool_id.clone(),
        build_evidence_id: prepared.contract.build_evidence_id.clone(),
        maximum_invocations: request.maximum_build_invocations,
        retry_delay: request.retry_delay,
    })?;
    let final_attempt =
        build.attempts.last().cloned().context("successful build has no attempt record")?;
    if final_attempt.failure_class.is_some() {
        bail!("successful build report ends in a failed attempt");
    }
    let image_sha256 = bijux_dna_infra::hash_file_sha256(&staging_path)
        .with_context(|| format!("hash staged image {}", staging_path.display()))?;
    let identity_evidence = verify_apptainer_tool_contract(
        &request.apptainer_executable,
        &staging_path,
        &image_sha256,
        &prepared.contract,
    )?;
    if clean_repository_head_commit(&request.repository_root)? != repository_commit {
        bail!("repository commit changed during canonical image build");
    }
    validate_filename_component(&prepared.contract.version, "catalog tool version")?;
    let final_path = roots.image.join(format!(
        "{}-{}-linux-amd64-{:.12}.sif",
        request.tool_id, prepared.contract.version, image_sha256
    ));
    publish_image_noclobber(&staging_path, &final_path, &image_sha256)?;
    let image_size_bytes = fs::metadata(&final_path)
        .with_context(|| format!("inspect canonical image {}", final_path.display()))?
        .len();
    Ok(PublishedCanonicalImage {
        base_image_reference,
        base_image_digest,
        builder_host,
        apptainer_version,
        build_command,
        final_path,
        image_sha256,
        image_size_bytes,
        identity_evidence,
        final_attempt,
    })
}

fn pinned_base_image_identity(definition_path: &Path) -> Result<(String, String)> {
    let definition = fs::read_to_string(definition_path)
        .with_context(|| format!("read rendered definition {}", definition_path.display()))?;
    let references = definition
        .lines()
        .filter_map(|line| line.trim().strip_prefix("From:").map(str::trim))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let [reference] = references.as_slice() else {
        bail!("rendered definition must contain exactly one non-empty From identity");
    };
    let Some((_, digest)) = reference.rsplit_once("@sha256:") else {
        bail!("rendered definition base image is not pinned by SHA-256");
    };
    if digest.len() != 64
        || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("rendered definition base image has an invalid SHA-256 digest");
    }
    Ok(((*reference).to_string(), digest.to_string()))
}

fn command_text(executable: &Path, args: &[&str], label: &str) -> Result<String> {
    let output =
        Command::new(executable).args(args).output().with_context(|| format!("execute {label}"))?;
    if !output.status.success() {
        bail!("{label} command failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let value = String::from_utf8(output.stdout)
        .with_context(|| format!("decode {label}"))?
        .trim()
        .to_string();
    if value.is_empty() {
        bail!("{label} command returned an empty identity");
    }
    Ok(value)
}

fn canonical_container_roots(
    study_root: &Path,
    repository_commit: &str,
    destination: CanonicalApptainerImageDestination,
) -> Result<CanonicalContainerRoots> {
    let containers = study_root.join("containers");
    let (image, evidence) = match destination {
        CanonicalApptainerImageDestination::Active => {
            (containers.join("apptainer"), containers.join("evidence"))
        }
        CanonicalApptainerImageDestination::CommitCampaign => (
            containers.join("staging").join(repository_commit),
            containers.join("evidence/image-campaigns").join(repository_commit),
        ),
    };
    let staging = image.join(".building");
    let build_logs = evidence.join("build-logs");
    for root in [&image, &evidence, &staging, &build_logs] {
        bijux_dna_infra::ensure_dir(root)
            .with_context(|| format!("create canonical container root {}", root.display()))?;
    }
    Ok(CanonicalContainerRoots { image, evidence, staging, build_logs })
}

fn validate_request(request: &CanonicalApptainerImagePublicationRequest) -> Result<()> {
    if request.architecture != "linux/amd64" {
        bail!("canonical Apptainer publication currently requires linux/amd64 architecture");
    }
    if request.repository_root == request.study_root {
        bail!("repository and study roots must be distinct");
    }
    validate_filename_component(&request.tool_id, "canonical image tool ID")
}

fn validate_filename_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("{label} is not a safe filename component");
    }
    Ok(())
}

fn publish_image_noclobber(staging: &Path, destination: &Path, digest: &str) -> Result<()> {
    match fs::hard_link(staging, destination) {
        Ok(()) => {
            fs::remove_file(staging)
                .with_context(|| format!("remove published staging image {}", staging.display()))?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = bijux_dna_infra::hash_file_sha256(destination).with_context(|| {
                format!("hash existing canonical image {}", destination.display())
            })?;
            if existing != digest {
                bail!("canonical image digest-prefix collision at {}", destination.display());
            }
            fs::remove_file(staging)
                .with_context(|| format!("remove duplicate staging image {}", staging.display()))?;
            Ok(())
        }
        Err(error) => {
            Err(error).with_context(|| format!("publish canonical image {}", destination.display()))
        }
    }
}

fn publish_evidence_noclobber(
    path: &Path,
    publication: &CanonicalApptainerImagePublication,
) -> Result<()> {
    match bijux_dna_infra::atomic_create_json(path, publication) {
        Ok(()) => Ok(()),
        Err(error) if path.is_file() => {
            let existing: CanonicalApptainerImagePublication = serde_json::from_slice(
                &fs::read(path)
                    .with_context(|| format!("read existing evidence {}", path.display()))?,
            )
            .with_context(|| format!("parse existing evidence {}", path.display()))?;
            if &existing == publication {
                Ok(())
            } else {
                bail!("immutable build evidence conflicts at {}: {error}", path.display())
            }
        }
        Err(error) => {
            Err(error).with_context(|| format!("publish build evidence {}", path.display()))
        }
    }
}

fn publish_identity_noclobber(path: &Path, identity: &ToolImageIdentityEvidence) -> Result<()> {
    match bijux_dna_infra::atomic_create_json(path, identity) {
        Ok(()) => Ok(()),
        Err(error) if path.is_file() => {
            let existing: ToolImageIdentityEvidence = serde_json::from_slice(
                &fs::read(path)
                    .with_context(|| format!("read existing identity {}", path.display()))?,
            )
            .with_context(|| format!("parse existing identity {}", path.display()))?;
            if &existing == identity {
                Ok(())
            } else {
                bail!("immutable image identity conflicts at {}: {error}", path.display())
            }
        }
        Err(error) => {
            Err(error).with_context(|| format!("publish image identity {}", path.display()))
        }
    }
}

fn portable_study_path(path: &Path, study_root: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(study_root)
        .with_context(|| format!("evidence path is outside study root: {}", path.display()))?;
    Ok(format!("study://{}", relative.to_string_lossy().replace('\\', "/")))
}
