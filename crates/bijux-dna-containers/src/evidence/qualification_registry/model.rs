//! Owner: bijux-dna-containers
//! Persisted qualification-run identities and their historical allocation audit.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const QUALIFICATION_RUN_REGISTRY_SCHEMA_V1: &str =
    "bijux.container.qualification_run_registry.v1";
pub const QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V2: &str =
    "bijux.container.qualification_run_id_audit.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunRegistry {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub historical_max_run_number: u64,
    pub tools: Vec<QualificationRunRegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationRunRegistryEntry {
    pub tool_id: String,
    pub support_disposition: String,
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
    pub retained_mapping_count: usize,
    pub unsupported_tool_count: usize,
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
    pub support_disposition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub identity_source: String,
    pub stage_id: String,
    pub input_role: String,
}
