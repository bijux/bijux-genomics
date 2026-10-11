//! Owner: bijux-dna-containers
//! Qualification registry composition of source acquisition, allocation and admission.

use super::admission::{
    validate_container_run_index, validate_historical_ownership,
    validate_qualification_run_registry,
};
use super::allocation::derive_registry_rows;
use super::audit::derive_registry_audit;
use super::history::{scan_attempt_ledger, scan_current_runs, scan_supplemental_history};
use super::identity::{
    parse_run_id, RUN_NAMESPACE_MAXIMUM, RUN_NAMESPACE_MINIMUM, RUN_NAMESPACE_NEXT,
};
use super::protocol::{derive_run_definitions, summarize_run_definitions};
use super::storage::{
    capture_audit_sources, load_qualification_run_registry_for_derivation,
    load_registry_observation,
};
use super::{
    ContainerRunIndex, DerivedQualificationRunRegistry, QualificationRegistryDerivationRequest,
    QualificationRunNamespacePolicy, QualificationRunRegistry, CONTAINER_RUN_INDEX_SCHEMA_V1,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V3,
};
use crate::evidence::tool_contract::load_container_catalog;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Derive one stable run identity per runnable tool without reusing historical IDs.
///
/// # Errors
/// Returns an error when catalog counts, current final runs, historical attempts, or run-ID
/// ownership are malformed or ambiguous.
pub fn derive_qualification_run_registry(
    request: &QualificationRegistryDerivationRequest,
) -> Result<DerivedQualificationRunRegistry> {
    let catalog = load_container_catalog(&request.repository_root)?;
    let catalog_path = request.repository_root.join("containers/contracts/catalog.toml");
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let mut historical = BTreeMap::new();
    let current = scan_current_runs(&request.current_runs_root, &mut historical)?;
    scan_attempt_ledger(&request.attempt_ledger_path, &mut historical)?;
    let tombstones =
        scan_supplemental_history(&request.supplemental_history_path, &mut historical)?;
    let historical_max_run_number = historical.keys().next_back().copied().unwrap_or(0).max(
        tombstones
            .iter()
            .map(|tombstone| parse_run_id(&tombstone.run_id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(0),
    );
    let existing = request
        .existing_registry_path
        .as_deref()
        .filter(|path| path.is_file())
        .map(load_qualification_run_registry_for_derivation)
        .transpose()?;
    let reservations = existing
        .as_ref()
        .map(|registry| {
            registry
                .tools
                .iter()
                .filter_map(|row| {
                    row.run_id.as_deref().map(|run_id| (row.tool_id.as_str(), run_id))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let catalog_tools = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<BTreeSet<_>>();
    if current.keys().any(|tool| !catalog_tools.contains(tool.as_str())) {
        bail!("current qualification runs contain a tool absent from the catalog");
    }

    let mut derived_rows =
        derive_registry_rows(&catalog, &current, &reservations, historical_max_run_number)?;
    let runs = derive_run_definitions(
        &catalog,
        &derived_rows.entries,
        &current,
        &historical,
        &tombstones,
    )?;
    let registry = QualificationRunRegistry {
        schema_version: QUALIFICATION_RUN_REGISTRY_SCHEMA_V3.to_string(),
        catalog_sha256: catalog_sha256.clone(),
        historical_max_run_number: existing
            .as_ref()
            .map_or(historical_max_run_number, |registry| registry.historical_max_run_number),
        namespace: QualificationRunNamespacePolicy {
            minimum: RUN_NAMESPACE_MINIMUM,
            maximum: RUN_NAMESPACE_MAXIMUM,
            next_run_number: RUN_NAMESPACE_NEXT,
            allocation_policy: "monotonic_only".to_string(),
            identity_policy: "existing_ids_immutable_attempts_reuse_run_id".to_string(),
        },
        summary: summarize_run_definitions(&runs),
        tools: std::mem::take(&mut derived_rows.entries),
        runs,
    };
    validate_qualification_run_registry(&catalog, &registry)?;
    validate_historical_ownership(&registry, &historical, &tombstones)?;
    let audit_sources = capture_audit_sources(request)?;
    let audit = derive_registry_audit(
        &audit_sources,
        catalog_sha256,
        &historical,
        tombstones,
        historical_max_run_number,
        &registry,
        derived_rows,
    );
    Ok(DerivedQualificationRunRegistry { registry, audit })
}

/// Derive the complete paper-facing index from the canonical checked-in registry.
///
/// # Errors
/// Returns an error when the source registry, catalog closure, path, or index identity is invalid.
pub fn derive_container_run_index(
    repository_root: &Path,
    registry_path: &Path,
) -> Result<ContainerRunIndex> {
    let expected = repository_root.join("containers/contracts/qualification-runs.toml");
    if registry_path != expected {
        bail!("container run index requires the canonical qualification registry");
    }
    let observation = load_registry_observation(repository_root, registry_path)?;
    let registry = observation.registry;
    let index = ContainerRunIndex {
        schema_version: CONTAINER_RUN_INDEX_SCHEMA_V1.to_string(),
        source_registry: observation.source,
        catalog_sha256: registry.catalog_sha256,
        historical_max_run_number: registry.historical_max_run_number,
        namespace: registry.namespace,
        summary: registry.summary,
        runs: registry.runs,
    };
    validate_container_run_index(&index)?;
    Ok(index)
}
