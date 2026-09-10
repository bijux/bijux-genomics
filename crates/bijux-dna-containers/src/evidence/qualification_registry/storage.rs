//! Owner: bijux-dna-containers
//! Qualification registry filesystem acquisition and digest-bound source observations.

use super::admission::validate_qualification_run_registry;
use super::audit::RegistryAuditSources;
use super::legacy::decode_registry;
use super::QualificationRegistryDerivationRequest;
use super::{ContainerRunIndexSource, QualificationRunRegistry};
use crate::evidence::tool_contract::load_container_catalog;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

/// Load and validate the governed qualification registry against its current catalog.
/// # Errors
/// Rejects unreadable, malformed, stale or contradictory registry observations.
pub fn load_qualification_run_registry(
    repository_root: &Path,
    registry_path: &Path,
) -> Result<QualificationRunRegistry> {
    Ok(load_registry_observation(repository_root, registry_path)?.registry)
}

pub(super) fn load_qualification_run_registry_for_derivation(
    registry_path: &Path,
) -> Result<QualificationRunRegistry> {
    let bytes = fs::read(registry_path)
        .with_context(|| format!("read qualification registry {}", registry_path.display()))?;
    decode_registry(&bytes, registry_path)
}

fn portable_repository_path(path: &Path, repository_root: &Path) -> Result<String> {
    Ok(format!(
        "repository://{}",
        path.canonicalize()?
            .strip_prefix(repository_root.canonicalize()?)
            .context("qualification registry input is outside the repository")?
            .to_string_lossy()
            .replace('\\', "/")
    ))
}

pub(super) struct RegistryObservation {
    pub(super) registry: QualificationRunRegistry,
    pub(super) source: ContainerRunIndexSource,
}

pub(super) fn load_registry_observation(
    repository_root: &Path,
    registry_path: &Path,
) -> Result<RegistryObservation> {
    let bytes = fs::read(registry_path)
        .with_context(|| format!("read qualification registry {}", registry_path.display()))?;
    let registry = decode_registry(&bytes, registry_path)?;
    let catalog = load_container_catalog(repository_root)?;
    validate_qualification_run_registry(&catalog, &registry)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("containers/contracts/catalog.toml"),
    )?;
    if registry.catalog_sha256 != catalog_sha256 {
        bail!("qualification registry catalog identity is stale");
    }
    Ok(RegistryObservation {
        registry,
        source: ContainerRunIndexSource {
            uri: "repo://containers/contracts/qualification-runs.toml".to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
            size_bytes: u64::try_from(bytes.len()).context("qualification registry exceeds u64")?,
        },
    })
}

pub(super) fn capture_audit_sources(
    request: &QualificationRegistryDerivationRequest,
) -> Result<RegistryAuditSources> {
    Ok(RegistryAuditSources {
        current_runs: portable_repository_path(
            &request.current_runs_root,
            &request.repository_root,
        )?,
        attempt_ledger: portable_repository_path(
            &request.attempt_ledger_path,
            &request.repository_root,
        )?,
        supplemental_history: portable_repository_path(
            &request.supplemental_history_path,
            &request.repository_root,
        )?,
    })
}
