use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    SecurityProbeArtifact, SecurityProbePublication, SecurityProbeStatus, SecurityRuntimeReport,
    SecurityStructuralReport, ToolSecurityProbe, SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityProbeAuthority {
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub inventory_sha256: String,
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
    validate_publication_identity(&publication, authority)?;

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
    Ok(AdmittedSecurityProbeReports { structural, runtime })
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

fn validate_publication_identity(
    publication: &SecurityProbePublication,
    authority: &SecurityProbeAuthority,
) -> Result<()> {
    if publication.schema_version != SECURITY_PROBE_PUBLICATION_SCHEMA_V1 {
        bail!("unsupported security probe publication schema");
    }
    if publication.repository_commit != authority.repository_commit
        || publication.catalog_sha256 != authority.catalog_sha256
        || publication.inventory_sha256 != authority.inventory_sha256
        || publication.scanner_completion_sha256 != authority.scanner_completion_sha256
        || publication.database_completion_sha256 != authority.database_completion_sha256
    {
        bail!("security probe publication authority differs from the admitted authority");
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
