//! Owner: bijux-dna-containers
//! Persisted qualification-run identities and their historical allocation audit.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::evidence::tool_contract::{ContainerArtifactDelivery, ContainerProductSupport};

pub const QUALIFICATION_RUN_REGISTRY_SCHEMA_V3: &str =
    "bijux.container.qualification_run_registry.v3";
pub const QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V3: &str =
    "bijux.container.qualification_run_id_audit.v3";
pub const CONTAINER_RUN_INDEX_SCHEMA_V1: &str = "bijux.container.run_index.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerRunIndexSource {
    pub uri: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerRunIndex {
    pub schema_version: String,
    pub source_registry: ContainerRunIndexSource,
    pub catalog_sha256: String,
    pub historical_max_run_number: u64,
    pub namespace: QualificationRunNamespacePolicy,
    pub summary: QualificationRunRegistrySummary,
    pub runs: Vec<QualificationRunDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunRegistry {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub historical_max_run_number: u64,
    pub namespace: QualificationRunNamespacePolicy,
    pub summary: QualificationRunRegistrySummary,
    pub tools: Vec<QualificationRunRegistryEntry>,
    pub runs: Vec<QualificationRunDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunNamespacePolicy {
    pub minimum: u64,
    pub maximum: u64,
    pub next_run_number: u64,
    pub allocation_policy: String,
    pub identity_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunRegistrySummary {
    pub run_count: usize,
    pub campaign_count: usize,
    pub active_count: usize,
    pub allocated_count: usize,
    pub historical_count: usize,
    pub reserved_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualificationRunState {
    Campaign,
    Active,
    Allocated,
    Historical,
    Reserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunCitationRecord {
    pub citation_key: String,
    pub protocol_version: String,
    pub registry_anchor: String,
    pub evidence_uris: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunDefinition {
    pub run_id: String,
    pub title: String,
    pub purpose: String,
    pub owner: String,
    pub operation: String,
    pub stage_id: String,
    pub tool_ids: Vec<String>,
    pub contract_ids: Vec<String>,
    pub corpus_roles: Vec<String>,
    pub acceptance_policy: String,
    pub state: QualificationRunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_reason: Option<String>,
    pub supersedes_run_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by_run_id: Option<String>,
    pub citation: QualificationRunCitationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunRegistryEntry {
    pub tool_id: String,
    pub product_support: ContainerProductSupport,
    pub artifact_delivery: ContainerArtifactDelivery,
    pub redistribution_permitted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub stage_id: String,
    pub input_role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceptance_adapter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunIdAudit {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub current_run_root: String,
    pub attempt_ledger: String,
    pub supplemental_history: String,
    pub observed_historical_run_count: usize,
    pub reserved_run_count: usize,
    pub historical_max_run_number: u64,
    pub supported_tool_count: usize,
    pub redistributable_tool_count: usize,
    pub external_artifact_tool_count: usize,
    pub allocated_mapping_count: usize,
    pub historical_identities: Vec<QualificationHistoricalRunIdentity>,
    pub reserved_run_ids: Vec<QualificationRunIdTombstone>,
    pub rows: Vec<QualificationRunIdAuditRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationHistoricalRunIdentity {
    pub run_id: String,
    pub tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunIdTombstone {
    pub run_id: String,
    pub disposition: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunIdAuditRow {
    pub tool_id: String,
    pub product_support: ContainerProductSupport,
    pub artifact_delivery: ContainerArtifactDelivery,
    pub redistribution_permitted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub identity_source: String,
    pub stage_id: String,
    pub input_role: String,
}

/// Repository locations consumed by qualification registry derivation.
#[derive(Debug, Clone)]
pub struct QualificationRegistryDerivationRequest {
    pub repository_root: std::path::PathBuf,
    pub current_runs_root: std::path::PathBuf,
    pub attempt_ledger_path: std::path::PathBuf,
    pub supplemental_history_path: std::path::PathBuf,
    pub existing_registry_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DerivedQualificationRunRegistry {
    pub registry: QualificationRunRegistry,
    pub audit: QualificationRunIdAudit,
}
