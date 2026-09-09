//! Owner: benchmark repository acquisition.
//! Acquisition of content-bound container qualification publications and sources.
use anyhow::{Context, Result};
use bijux_dna_policies::{
    ContainerReleaseEvidencePublication, ContainerSecurityEvidencePublication,
    QualificationEvidenceSource,
};
use std::fs;
use std::path::Path;

pub(crate) const INVENTORY_PATH: &str = "containers/reports/image-inventory.json";
pub(crate) const RUNTIME_PATH: &str = "containers/reports/runtime-verification.json";
pub(crate) const RUNS_PATH: &str = "containers/reports/run-evidence.json";
pub(crate) const SECURITY_PATH: &str = "containers/reports/security-evidence.json";

pub(crate) fn load_release_publication(
    path: &Path,
) -> Result<(ContainerReleaseEvidencePublication, QualificationEvidenceSource)> {
    let bytes =
        std::fs::read(path).with_context(|| format!("read release evidence {}", path.display()))?;
    let sha256 = bijux_dna_infra::hash_bytes_sha256(&bytes);
    let publication = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse release evidence {}", path.display()))?;
    let source =
        QualificationEvidenceSource { uri: format!("evidence://release/{sha256}"), sha256 };
    Ok((publication, source))
}

pub(crate) fn load_security_publication(
    repository_root: &Path,
) -> Result<Option<ContainerSecurityEvidencePublication>> {
    let path = repository_root.join(SECURITY_PATH);
    if !path.is_file() {
        return Ok(None);
    }
    serde_json::from_slice(&std::fs::read(&path)?)
        .with_context(|| format!("parse security evidence {}", path.display()))
        .map(Some)
}

pub(crate) fn repository_source(
    repository_root: &Path,
    relative: &str,
) -> Result<QualificationEvidenceSource> {
    let bytes = fs::read(repository_root.join(relative))
        .with_context(|| format!("read qualification source {relative}"))?;
    Ok(QualificationEvidenceSource {
        uri: format!("repository://{relative}"),
        sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
    })
}
