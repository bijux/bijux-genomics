//! Current repository observations and immutable admission publication.
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::super::tool_contract::{
    catalog_identity_at_commit, load_container_catalog, repository_head_commit,
};
use super::super::EvidenceSourceIdentity;
use super::model::{
    ExternalArtifactAdmission, ExternalArtifactAdmissionStatus, ExternalArtifactCommandBinding,
    ExternalArtifactIdentity, ExternalArtifactVersionProbe, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2,
};

use super::admission::{
    decode_external_artifact_admission, require_timestamp, validate_external_artifact_admission,
    validate_persisted_version_probe, validate_sha256,
};

use super::probe::{
    probe_version, ExternalArtifactProbeTransport, NativeExternalArtifactProbeTransport,
};
use super::request::{ExternalArtifactAdmissionRequest, ExternalArtifactInput};

/// Admit an operator-provided artifact without copying it into repository or active image storage.
///
/// # Errors
/// Returns an error when catalog capability, source identity, access basis, checksum, version, or
/// no-clobber publication requirements are not satisfied.
pub fn admit_external_artifact(
    request: &ExternalArtifactAdmissionRequest<'_>,
    output: &Path,
) -> Result<ExternalArtifactAdmission> {
    admit_external_artifact_with_probe(request, output, &mut NativeExternalArtifactProbeTransport)
}

/// Admit an artifact using an explicit version-observation transport.
/// The same catalog, source, checksum, version and immutable-publication requirements apply
/// as in [`admit_external_artifact`]. Blocked/unavailable declarations never invoke the probe.
///
/// # Errors
/// Rejects invalid requests before probing and rejects unsuccessful or incompatible evidence
/// before publication. A transport error does not become a successful availability record.
pub fn admit_external_artifact_with_probe(
    request: &ExternalArtifactAdmissionRequest<'_>,
    output: &Path,
    transport: &mut dyn ExternalArtifactProbeTransport,
) -> Result<ExternalArtifactAdmission> {
    require_timestamp(request.recorded_at_utc)?;
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize external-artifact repository root")?;
    let distributable_image_root = request
        .distributable_image_root
        .canonicalize()
        .context("canonicalize distributable image root")?;
    let head = repository_head_commit(&repository_root)?;
    if head != request.repository_commit {
        bail!("external-artifact admission commit differs from repository HEAD");
    }
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)
        .context("hash external-artifact catalog")?;
    if catalog_identity_at_commit(&repository_root, request.repository_commit)? != catalog_sha256 {
        bail!("external-artifact catalog differs from the requested repository commit");
    }
    let catalog = load_container_catalog(&repository_root)?;
    let tool = catalog
        .tools
        .into_iter()
        .find(|tool| tool.id == request.tool_id)
        .with_context(|| format!("container catalog omits tool {}", request.tool_id))?;
    if !tool.is_product_supported()
        || !tool.permits_external_artifact()
        || tool.redistribution_permitted
        || !tool.external_artifact_supported
    {
        bail!("tool {} does not permit external artifact admission", request.tool_id);
    }
    if tool.apptainer_supported || tool.docker_arm64_supported {
        bail!("external artifact tool {} exposes a distributable image", request.tool_id);
    }

    let (lawful_access_basis, artifact, version_probe, blocker_reason) =
        match (request.availability, &request.artifact, request.blocker_reason) {
            (ExternalArtifactAdmissionStatus::Admitted, Some(input), None) => {
                let admitted = inspect_artifact(
                    &repository_root,
                    &distributable_image_root,
                    &tool,
                    input,
                    transport,
                )?;
                (
                    Some(input.lawful_access_basis.trim().to_string()),
                    Some(admitted.0),
                    Some(admitted.1),
                    None,
                )
            }
            (
                ExternalArtifactAdmissionStatus::BlockedExternalAccess
                | ExternalArtifactAdmissionStatus::Unavailable,
                None,
                Some(reason),
            ) if !reason.trim().is_empty() => (None, None, None, Some(reason.trim().to_string())),
            _ => bail!(
                "external artifact admission inputs do not match requested availability {:?}",
                request.availability
            ),
        };

    let admission = ExternalArtifactAdmission {
        schema_version: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2.to_string(),
        availability: request.availability,
        recorded_at_utc: request.recorded_at_utc.to_string(),
        repository_commit: request.repository_commit.to_string(),
        catalog: EvidenceSourceIdentity {
            path: "containers/contracts/catalog.toml".to_string(),
            sha256: catalog_sha256,
        },
        distributable_image_root: distributable_image_root.display().to_string(),
        tool_id: tool.id,
        version: tool.version,
        product_support: tool.product_support,
        artifact_delivery: tool.artifact_delivery,
        delivery_reason: tool.delivery_reason,
        stage_ids: tool.stage_ids,
        expected_binary: tool.expected_bin,
        version_command: tool.version_command,
        expected_version_regex: tool.expected_version_regex,
        redistribution_permitted: false,
        lawful_access_basis,
        artifact,
        version_probe,
        blocker_reason,
    };
    validate_external_artifact_admission(&admission)?;
    bijux_dna_infra::atomic_create_json(output, &admission)
        .context("publish external-artifact admission")?;
    Ok(admission)
}

/// Load and validate an external-artifact admission record.
///
/// # Errors
/// Returns an error when the document is malformed or violates lifecycle invariants.
pub fn load_external_artifact_admission(path: &Path) -> Result<ExternalArtifactAdmission> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 4 * 1024 * 1024)
        .context("read external-artifact admission")?;
    decode_external_artifact_admission(&bytes)
}

/// Reverify catalog and artifact identities without executing a stage command.
///
/// # Errors
/// Returns an error when catalog capability changed or an admitted artifact drifted.
pub fn verify_external_artifact_admission(
    repository_root: &Path,
    distributable_image_root: &Path,
    admission_path: &Path,
) -> Result<ExternalArtifactAdmission> {
    let admission = load_external_artifact_admission(admission_path)?;
    let repository_root =
        repository_root.canonicalize().context("canonicalize external-artifact repository root")?;
    let distributable_image_root = distributable_image_root
        .canonicalize()
        .context("canonicalize requested distributable image root")?;
    if Path::new(&admission.distributable_image_root) != distributable_image_root {
        bail!("external-artifact distributable image root changed");
    }
    let catalog_path = repository_root.join(&admission.catalog.path);
    let current_catalog_sha256 =
        bijux_dna_infra::hash_file_sha256(&catalog_path).context("hash current catalog")?;
    if current_catalog_sha256 != admission.catalog.sha256 {
        bail!("external-artifact catalog identity changed");
    }
    if catalog_identity_at_commit(&repository_root, &admission.repository_commit)?
        != admission.catalog.sha256
    {
        bail!("external-artifact repository commit does not contain the admitted catalog");
    }
    let catalog = load_container_catalog(&repository_root)?;
    let tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == admission.tool_id)
        .context("external-artifact tool no longer exists in catalog")?;
    if !tool.external_artifact_supported
        || tool.product_support != admission.product_support
        || tool.artifact_delivery != admission.artifact_delivery
        || tool.delivery_reason != admission.delivery_reason
        || tool.version != admission.version
        || tool.stage_ids != admission.stage_ids
        || tool.expected_bin != admission.expected_binary
        || tool.version_command != admission.version_command
        || tool.expected_version_regex != admission.expected_version_regex
    {
        bail!("external-artifact tool contract changed");
    }
    if let Some(artifact) = &admission.artifact {
        let path = Path::new(&artifact.path);
        let resolved = path.canonicalize().context("resolve current external artifact location")?;
        if resolved != path {
            bail!("external artifact canonical location changed");
        }
        if path.starts_with(&repository_root) {
            bail!("external artifact entered the product repository");
        }
        if path.starts_with(&distributable_image_root) {
            bail!("external artifact entered the distributable active image root");
        }
        if !tool.external_artifact_kinds.iter().any(|kind| kind == artifact.kind.as_str()) {
            bail!("external artifact kind is no longer permitted");
        }
        let metadata = path.metadata().context("inspect external artifact")?;
        if !metadata.is_file()
            || metadata.len() != artifact.size_bytes
            || bijux_dna_infra::hash_file_sha256(path).context("rehash external artifact")?
                != artifact.sha256
        {
            bail!("external artifact identity changed");
        }
        validate_persisted_version_probe(&admission, artifact)?;
    }
    Ok(admission)
}

/// Bind a stage command to a reverified operator artifact.
///
/// # Errors
/// Returns an error before execution when access is blocked, identity drifted, the stage is not
/// allowed, or the command does not invoke the cataloged binary.
pub fn bind_external_artifact_command(
    repository_root: &Path,
    distributable_image_root: &Path,
    admission_path: &Path,
    stage_id: &str,
    command: &[String],
    apptainer_executable: Option<&Path>,
) -> Result<ExternalArtifactCommandBinding> {
    let admission = verify_external_artifact_admission(
        repository_root,
        distributable_image_root,
        admission_path,
    )?;
    super::binding::bind_verified_external_artifact_command(
        admission,
        stage_id,
        command,
        apptainer_executable,
    )
}

fn inspect_artifact(
    repository_root: &Path,
    distributable_image_root: &Path,
    tool: &crate::evidence::tool_contract::ContainerCatalogTool,
    input: &ExternalArtifactInput<'_>,
    transport: &mut dyn ExternalArtifactProbeTransport,
) -> Result<(ExternalArtifactIdentity, ExternalArtifactVersionProbe)> {
    if input.lawful_access_basis.trim().is_empty() {
        bail!("external artifact requires a lawful access basis");
    }
    if !tool.external_artifact_kinds.iter().any(|kind| kind == input.kind.as_str()) {
        bail!("tool {} does not permit {} artifacts", tool.id, input.kind.as_str());
    }
    validate_sha256(input.expected_sha256, "external artifact checksum")?;
    let artifact_path = input.path.canonicalize().context("canonicalize external artifact")?;
    if artifact_path.starts_with(repository_root) {
        bail!("external artifact must remain outside the product repository");
    }
    if artifact_path.starts_with(distributable_image_root) {
        bail!("external artifact must remain outside the distributable active image root");
    }
    let metadata = artifact_path.metadata().context("inspect external artifact")?;
    if !metadata.is_file() {
        bail!("external artifact is not a regular file");
    }
    let sha256 =
        bijux_dna_infra::hash_file_sha256(&artifact_path).context("hash external artifact")?;
    if sha256 != input.expected_sha256.to_ascii_lowercase() {
        bail!("external artifact checksum does not match operator declaration");
    }
    let version_probe = probe_version(tool, input, &artifact_path, transport)?;
    Ok((
        ExternalArtifactIdentity {
            kind: input.kind,
            path: artifact_path.display().to_string(),
            sha256,
            size_bytes: metadata.len(),
        },
        version_probe,
    ))
}
