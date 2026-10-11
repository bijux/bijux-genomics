use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    SecurityFilesystemObservation, SecurityProbeArtifact, SecurityProbePublication,
    SecurityProbeStatus, SecurityRuntimeReport, SecurityStructuralReport, ToolSecurityProbe,
    SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityProbeAuthority {
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub inventory_sha256: String,
    pub scanner_completion_sha256: String,
    pub database_completion_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityProbeBaselineAuthority {
    pub catalog_sha256: String,
    pub scanner_completion_sha256: String,
    pub database_completion_sha256: String,
}

#[derive(Debug, Clone)]
pub struct AdmittedSecurityProbePublication {
    pub publication: SecurityProbePublication,
    pub publication_uri: String,
    pub publication_sha256: String,
    pub reports: BTreeMap<String, AdmittedSecurityProbeReports>,
    artifact_paths: BTreeMap<String, (String, PathBuf)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedSecurityProbeReports {
    pub structural: Option<SecurityStructuralReport>,
    pub runtime: Option<SecurityRuntimeReport>,
    pub raw_manifest: BTreeMap<String, String>,
}

impl AdmittedSecurityProbePublication {
    /// Read an admitted artifact and recheck its digest at the point of use.
    ///
    /// # Errors
    /// Returns an error when the artifact was not admitted or its bytes changed after admission.
    pub fn read_artifact(&self, artifact: &SecurityProbeArtifact) -> Result<Vec<u8>> {
        let (digest, path) = self
            .artifact_paths
            .get(&artifact.uri)
            .with_context(|| format!("security artifact was not admitted: {}", artifact.uri))?;
        if digest != &artifact.sha256 {
            bail!("security artifact identity differs from its admitted identity");
        }
        let bytes = fs::read(path)
            .with_context(|| format!("read admitted security artifact {}", path.display()))?;
        if bijux_dna_infra::hash_bytes_sha256(&bytes) != artifact.sha256 {
            bail!("admitted security artifact changed at {}", path.display());
        }
        Ok(bytes)
    }
}

/// Admit one raw exact-image security publication and all of its referenced artifacts.
///
/// # Errors
/// Returns an error for authority drift, malformed tool states, unsafe artifact URIs, missing
/// artifacts, or any publication or artifact digest mismatch.
pub fn admit_security_probe_publication(
    publication_path: &Path,
    study_root: &Path,
    authority: &SecurityProbeAuthority,
) -> Result<AdmittedSecurityProbePublication> {
    validate_authority(authority)?;
    admit_publication(publication_path, study_root, |publication| {
        validate_publication_identity(publication, authority)
    })
}

/// Admit a prior raw security publication for unchanged-image evidence reuse.
///
/// Baselines may come from an earlier repository and inventory identity, but must use the exact
/// current catalog, scanner toolchain, and vulnerability database. Every referenced artifact is
/// admitted and digest-checked identically to a current publication.
///
/// # Errors
/// Returns an error for stable-authority drift, malformed source identities, unsafe artifact
/// URIs, missing artifacts, or any publication or artifact digest mismatch.
pub fn admit_security_probe_baseline(
    publication_path: &Path,
    study_root: &Path,
    authority: &SecurityProbeBaselineAuthority,
) -> Result<AdmittedSecurityProbePublication> {
    validate_baseline_authority(authority)?;
    admit_publication(publication_path, study_root, |publication| {
        validate_baseline_identity(publication, authority)
    })
}

fn admit_publication(
    publication_path: &Path,
    study_root: &Path,
    validate_identity: impl FnOnce(&SecurityProbePublication) -> Result<()>,
) -> Result<AdmittedSecurityProbePublication> {
    let study_root = study_root
        .canonicalize()
        .with_context(|| format!("canonicalize study root {}", study_root.display()))?;
    if !fs::symlink_metadata(publication_path)?.file_type().is_file() {
        bail!("security publication must be a regular file, not a symlink");
    }
    let publication_path = publication_path.canonicalize().with_context(|| {
        format!("canonicalize security publication {}", publication_path.display())
    })?;
    if !publication_path.is_file() || !publication_path.starts_with(&study_root) {
        bail!("security publication is not a file under the governed study root");
    }
    let publication_bytes = fs::read(&publication_path)?;
    let publication: SecurityProbePublication = serde_json::from_slice(&publication_bytes)
        .context("parse raw security probe publication")?;
    validate_identity(&publication)?;

    let mut artifact_paths = BTreeMap::new();
    let mut reports = BTreeMap::new();
    for (tool_id, tool) in &publication.tools {
        validate_tool(tool_id, tool)?;
        for artifact in tool_artifacts(tool) {
            admit_artifact(&study_root, artifact, &mut artifact_paths)?;
        }
        reports.insert(tool_id.clone(), admit_reports(tool, &study_root, &mut artifact_paths)?);
    }
    let relative = publication_path
        .strip_prefix(&study_root)
        .context("security publication escaped the governed study root")?;
    let relative = relative.to_str().context("security publication URI is not valid UTF-8")?;
    Ok(AdmittedSecurityProbePublication {
        publication,
        publication_uri: format!("study://{relative}"),
        publication_sha256: bijux_dna_infra::hash_bytes_sha256(&publication_bytes),
        reports,
        artifact_paths,
    })
}

fn admit_reports(
    tool: &ToolSecurityProbe,
    study_root: &Path,
    artifacts: &mut BTreeMap<String, (String, PathBuf)>,
) -> Result<AdmittedSecurityProbeReports> {
    let structural = tool
        .structural_report
        .as_ref()
        .map(|artifact| read_json_artifact::<SecurityStructuralReport>(artifact, artifacts))
        .transpose()?;
    if let Some(report) = &structural {
        validate_structural_report(report)?;
    }
    let runtime = tool
        .runtime_report
        .as_ref()
        .map(|artifact| read_json_artifact::<SecurityRuntimeReport>(artifact, artifacts))
        .transpose()?;
    if let Some(report) = &runtime {
        for artifact in [
            &report.network_isolated.stdout,
            &report.network_isolated.stderr,
            &report.non_root_uid.stdout,
            &report.non_root_uid.stderr,
            &report.system_read_only.stdout,
            &report.system_read_only.stderr,
        ] {
            admit_artifact(study_root, artifact, artifacts)?;
        }
    }
    let raw_manifest =
        read_json_artifact::<BTreeMap<String, String>>(&tool.raw_manifest, artifacts)?;
    admit_raw_manifest(tool, runtime.as_ref(), &raw_manifest, study_root, artifacts)?;
    Ok(AdmittedSecurityProbeReports { structural, runtime, raw_manifest })
}

fn admit_raw_manifest(
    tool: &ToolSecurityProbe,
    runtime: Option<&SecurityRuntimeReport>,
    manifest: &BTreeMap<String, String>,
    study_root: &Path,
    artifacts: &mut BTreeMap<String, (String, PathBuf)>,
) -> Result<()> {
    if manifest.is_empty() {
        bail!("security raw manifest is empty for {}", tool.tool_id);
    }
    let manifest_relative = tool
        .raw_manifest
        .uri
        .strip_prefix("study://")
        .context("security raw manifest URI is not rooted at study://")?;
    let manifest_parent =
        Path::new(manifest_relative).parent().context("security raw manifest has no parent")?;
    let mut manifested = BTreeMap::new();
    for (name, digest) in manifest {
        if !safe_file_name(name) || matches!(name.as_str(), "raw-manifest.json" | "result.json") {
            bail!("security raw manifest contains an unsafe path for {}", tool.tool_id);
        }
        validate_sha256(digest, "raw manifest entry")?;
        let relative = manifest_parent.join(name);
        let relative = relative.to_str().context("security raw manifest path is not UTF-8")?;
        let artifact =
            SecurityProbeArtifact { uri: format!("study://{relative}"), sha256: digest.clone() };
        admit_artifact(study_root, &artifact, artifacts)?;
        manifested.insert(artifact.uri, artifact.sha256);
    }
    for artifact in tool_artifacts(tool).filter(|artifact| artifact.uri != tool.raw_manifest.uri) {
        if manifested.get(&artifact.uri) != Some(&artifact.sha256) {
            bail!("security raw manifest omits or changes referenced artifact {}", artifact.uri);
        }
    }
    if let Some(report) = runtime {
        for artifact in [
            &report.network_isolated.stdout,
            &report.network_isolated.stderr,
            &report.non_root_uid.stdout,
            &report.non_root_uid.stderr,
            &report.system_read_only.stdout,
            &report.system_read_only.stderr,
        ] {
            if manifested.get(&artifact.uri) != Some(&artifact.sha256) {
                bail!(
                    "security raw manifest omits or changes referenced artifact {}",
                    artifact.uri
                );
            }
        }
    }
    Ok(())
}

fn read_json_artifact<T: serde::de::DeserializeOwned>(
    artifact: &SecurityProbeArtifact,
    artifacts: &BTreeMap<String, (String, PathBuf)>,
) -> Result<T> {
    let (_, path) =
        artifacts.get(&artifact.uri).context("structured security artifact was not admitted")?;
    serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse structured security artifact {}", artifact.uri))
}

fn validate_structural_report(report: &SecurityStructuralReport) -> Result<()> {
    for (label, paths) in [
        ("SUID/SGID", report.suid_sgid_paths.as_slice()),
        ("world-writable", report.world_writable_paths.as_slice()),
    ] {
        if !paths.windows(2).all(|pair| pair[0] < pair[1])
            || paths
                .iter()
                .any(|path| !path.starts_with('/') || path.contains('\n') || path.contains('\r'))
        {
            bail!("security {label} paths are unsafe, duplicated, or unsorted");
        }
    }
    validate_structural_observations(
        "SUID/SGID",
        &report.suid_sgid_paths,
        &report.suid_sgid_observations,
        0o6000,
    )?;
    validate_structural_observations(
        "world-writable",
        &report.world_writable_paths,
        &report.world_writable_observations,
        0o002,
    )?;
    Ok(())
}

fn validate_structural_observations(
    label: &str,
    paths: &[String],
    observations: &[SecurityFilesystemObservation],
    required_mode: u32,
) -> Result<()> {
    if observations.is_empty() {
        return Ok(());
    }
    if observations.len() != paths.len()
        || observations.iter().zip(paths).any(|(observation, path)| {
            observation.path != *path
                || observation.mode > 0o7777
                || observation.mode & required_mode == 0
        })
    {
        bail!("security {label} observations differ from their path projection");
    }
    Ok(())
}

fn validate_authority(authority: &SecurityProbeAuthority) -> Result<()> {
    validate_commit(&authority.repository_commit)?;
    for (label, value) in [
        ("catalog", authority.catalog_sha256.as_str()),
        ("inventory", authority.inventory_sha256.as_str()),
        ("scanner completion", authority.scanner_completion_sha256.as_str()),
        ("database completion", authority.database_completion_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    Ok(())
}

fn validate_baseline_authority(authority: &SecurityProbeBaselineAuthority) -> Result<()> {
    for (label, value) in [
        ("catalog", authority.catalog_sha256.as_str()),
        ("scanner completion", authority.scanner_completion_sha256.as_str()),
        ("database completion", authority.database_completion_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    Ok(())
}

fn validate_publication_identity(
    publication: &SecurityProbePublication,
    authority: &SecurityProbeAuthority,
) -> Result<()> {
    validate_publication_shape(publication)?;
    if publication.repository_commit != authority.repository_commit
        || publication.catalog_sha256 != authority.catalog_sha256
        || publication.inventory_sha256 != authority.inventory_sha256
        || publication.scanner_completion_sha256 != authority.scanner_completion_sha256
        || publication.database_completion_sha256 != authority.database_completion_sha256
    {
        bail!("security probe publication authority differs from the admitted authority");
    }
    Ok(())
}

fn validate_baseline_identity(
    publication: &SecurityProbePublication,
    authority: &SecurityProbeBaselineAuthority,
) -> Result<()> {
    validate_publication_shape(publication)?;
    validate_commit(&publication.repository_commit)?;
    validate_sha256(&publication.inventory_sha256, "inventory")?;
    if publication.catalog_sha256 != authority.catalog_sha256
        || publication.scanner_completion_sha256 != authority.scanner_completion_sha256
        || publication.database_completion_sha256 != authority.database_completion_sha256
    {
        bail!("security probe baseline stable authority differs from the admitted authority");
    }
    Ok(())
}

fn validate_publication_shape(publication: &SecurityProbePublication) -> Result<()> {
    if publication.schema_version != SECURITY_PROBE_PUBLICATION_SCHEMA_V1 {
        bail!("unsupported security probe publication schema");
    }
    if let Some(digest) = &publication.reused_publication_sha256 {
        validate_sha256(digest, "reused security publication")?;
    }
    if publication.tools.is_empty() {
        bail!("security probe publication contains no tools");
    }
    Ok(())
}

fn validate_tool(key: &str, tool: &ToolSecurityProbe) -> Result<()> {
    if key != tool.tool_id
        || !safe_identifier(key)
        || tool.version.trim().is_empty()
        || tool.version != tool.version.trim()
        || tool.version.chars().any(char::is_control)
    {
        bail!("security probe tool identity is malformed for {key}");
    }
    for (label, value) in [
        ("image", tool.image_sha256.as_str()),
        ("tool contract", tool.tool_contract_sha256.as_str()),
        ("build evidence", tool.build_evidence_id.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    match tool.status {
        SecurityProbeStatus::Collected => {
            if tool.error.is_some()
                || tool.sbom.is_none()
                || tool.vulnerability_report.is_none()
                || tool.secret_report.is_none()
                || tool.signature_stdout.is_none()
                || tool.signature_stderr.is_none()
                || tool.structural_report.is_none()
                || tool.runtime_report.is_none()
            {
                bail!("collected security probe is incomplete for {key}");
            }
        }
        SecurityProbeStatus::Failed => {
            if tool.signature_verified
                || tool.error.as_deref().is_none_or(|error| error.trim().is_empty())
            {
                bail!("failed security probe lacks a valid failure record for {key}");
            }
        }
    }
    Ok(())
}

fn tool_artifacts(tool: &ToolSecurityProbe) -> impl Iterator<Item = &SecurityProbeArtifact> {
    [
        tool.sbom.as_ref(),
        tool.vulnerability_report.as_ref(),
        tool.secret_report.as_ref(),
        tool.signature_stdout.as_ref(),
        tool.signature_stderr.as_ref(),
        tool.structural_report.as_ref(),
        tool.runtime_report.as_ref(),
        Some(&tool.raw_manifest),
    ]
    .into_iter()
    .flatten()
}

fn admit_artifact(
    study_root: &Path,
    artifact: &SecurityProbeArtifact,
    admitted: &mut BTreeMap<String, (String, PathBuf)>,
) -> Result<()> {
    validate_sha256(&artifact.sha256, "artifact")?;
    let relative = artifact
        .uri
        .strip_prefix("study://")
        .context("security artifact URI is not rooted at study://")?;
    if relative.is_empty()
        || !Path::new(relative)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        bail!("security artifact URI contains an unsafe path: {}", artifact.uri);
    }
    let candidate = study_root.join(relative);
    if !fs::symlink_metadata(&candidate)?.file_type().is_file() {
        bail!("security artifact must be a regular file, not a symlink");
    }
    let path = candidate
        .canonicalize()
        .with_context(|| format!("canonicalize security artifact {}", artifact.uri))?;
    if !path.is_file() || !path.starts_with(study_root) {
        bail!("security artifact is not a file under the governed study root");
    }
    let observed = bijux_dna_infra::hash_file_sha256(&path)?;
    if observed != artifact.sha256 {
        bail!("security artifact digest differs for {}", artifact.uri);
    }
    match admitted.get(&artifact.uri) {
        Some((digest, existing_path)) if digest != &artifact.sha256 || existing_path != &path => {
            bail!("security artifact URI has conflicting admitted identities");
        }
        Some(_) => {}
        None => {
            admitted.insert(artifact.uri.clone(), (artifact.sha256.clone(), path));
        }
    }
    Ok(())
}

fn validate_commit(value: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("security probe repository commit is not a full Git identity");
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("security probe {label} is not a SHA-256 digest");
    }
    Ok(())
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn safe_file_name(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().count() == 1
        && Path::new(value).components().all(|component| matches!(component, Component::Normal(_)))
}
