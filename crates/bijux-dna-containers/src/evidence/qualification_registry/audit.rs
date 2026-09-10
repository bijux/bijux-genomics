//! Owner: bijux-dna-containers
//! Qualification registry pure audit projection over admitted observations and captured source paths.

use super::allocation::DerivedRegistryRows;
use super::{
    QualificationHistoricalRunIdentity, QualificationRunIdAudit, QualificationRunIdTombstone,
    QualificationRunRegistry, QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V3,
};
use crate::evidence::tool_contract::{ContainerArtifactDelivery, ContainerProductSupport};
use std::collections::BTreeMap;

pub(super) struct RegistryAuditSources {
    pub(super) current_runs: String,
    pub(super) attempt_ledger: String,
    pub(super) supplemental_history: String,
}

pub(super) fn derive_registry_audit(
    sources: &RegistryAuditSources,
    catalog_sha256: String,
    historical: &BTreeMap<u64, String>,
    tombstones: Vec<QualificationRunIdTombstone>,
    historical_max_run_number: u64,
    registry: &QualificationRunRegistry,
    derived_rows: DerivedRegistryRows,
) -> QualificationRunIdAudit {
    QualificationRunIdAudit {
        schema_version: QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V3.to_string(),
        catalog_sha256,
        current_run_root: sources.current_runs.clone(),
        attempt_ledger: sources.attempt_ledger.clone(),
        supplemental_history: sources.supplemental_history.clone(),
        observed_historical_run_count: historical.len(),
        reserved_run_count: tombstones.len(),
        historical_max_run_number,
        supported_tool_count: registry
            .tools
            .iter()
            .filter(|row| row.product_support == ContainerProductSupport::Supported)
            .count(),
        redistributable_tool_count: delivery_count(
            registry,
            ContainerArtifactDelivery::RedistributableImage,
        ),
        external_artifact_tool_count: delivery_count(
            registry,
            ContainerArtifactDelivery::ExternalArtifact,
        ),
        allocated_mapping_count: derived_rows.allocated_mapping_count,
        historical_identities: historical
            .iter()
            .map(|(number, tool_id)| QualificationHistoricalRunIdentity {
                run_id: format!("run-{number}"),
                tool_id: tool_id.clone(),
            })
            .collect(),
        reserved_run_ids: tombstones,
        rows: derived_rows.audit_rows,
    }
}

fn delivery_count(
    registry: &QualificationRunRegistry,
    delivery: ContainerArtifactDelivery,
) -> usize {
    registry.tools.iter().filter(|row| row.artifact_delivery == delivery).count()
}
