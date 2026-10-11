use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::qualification::{EvidenceVerdict, SupportDisposition};

pub const CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1: &str = "bijux.container.security_evidence.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerSecurityEvidencePublication {
    #[schemars(schema_with = "security_evidence_schema_version")]
    pub schema_version: String,
    pub evaluated_at_utc: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub inventory_sha256: String,
    pub policy_sha256: String,
    pub raw_evidence: SecurityArtifactIdentity,
    pub scanner: VulnerabilityScannerIdentity,
    pub tools: BTreeMap<String, ToolSecurityEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityArtifactIdentity {
    pub uri: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VulnerabilityScannerIdentity {
    pub name: String,
    pub version: String,
    pub database: VulnerabilityDatabaseIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VulnerabilityDatabaseIdentity {
    pub status: VulnerabilityDatabaseStatus,
    pub updated_at_utc: Option<String>,
    pub sha256: Option<String>,
    pub maximum_age_days: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VulnerabilityDatabaseStatus {
    Current,
    Stale,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolSecurityEvidence {
    pub tool_id: String,
    pub version: String,
    pub support_disposition: SupportDisposition,
    pub image_sha256: Option<String>,
    pub tool_contract_sha256: Option<String>,
    pub build_evidence_id: Option<String>,
    pub source_identity: Option<SecuritySourceIdentity>,
    pub structural: Option<StructuralSecurityEvidence>,
    pub sbom: Option<SbomEvidence>,
    pub vulnerabilities: Option<VulnerabilityEvidence>,
    pub secret_scan: Option<SecretScanEvidence>,
    pub runtime_policy: Option<RuntimeSecurityEvidence>,
    #[serde(default)]
    pub waivers: Vec<VulnerabilityWaiver>,
    pub verdict: EvidenceVerdict,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SecuritySourceIdentity {
    SourceSha256(String),
    PinnedCommit(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StructuralSecurityEvidence {
    pub inspection_sha256: String,
    pub signature: ImageSignatureDisposition,
    pub signature_evidence_sha256: String,
    #[serde(default)]
    pub suid_sgid_paths: Vec<String>,
    #[serde(default)]
    pub world_writable_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageSignatureDisposition {
    SignedVerified,
    UnsignedPolicyAccepted,
    Invalid,
    MissingPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SbomEvidence {
    pub format: SbomFormat,
    pub sha256: String,
    pub generator: String,
    pub generator_version: String,
    pub source_image_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SbomFormat {
    CycloneDxJson,
    SpdxJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VulnerabilityEvidence {
    pub scanner: String,
    pub scanner_version: String,
    pub database_sha256: String,
    pub report_sha256: String,
    pub severity_counts: VulnerabilitySeverityCounts,
    #[serde(default)]
    pub findings: Vec<VulnerabilityFinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VulnerabilitySeverityCounts {
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
    pub unknown: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VulnerabilityFinding {
    pub id: String,
    pub severity: VulnerabilitySeverity,
    pub package: String,
    pub installed_version: String,
    #[serde(default)]
    pub fixed_versions: Vec<String>,
    pub waiver_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VulnerabilityWaiver {
    pub id: String,
    pub vulnerability_id: String,
    pub rationale: String,
    pub expires_on: String,
    pub approval_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecretScanEvidence {
    pub scanner: String,
    pub scanner_version: String,
    pub report_sha256: String,
    pub finding_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSecurityEvidence {
    pub network_policy: RuntimeAccessPolicy,
    pub writable_mount_policy: RuntimeAccessPolicy,
    pub non_root_uid: u32,
    pub non_root_execution_passed: bool,
    pub probe_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAccessPolicy {
    Denied,
    Allowed,
    Unverified,
}

fn security_evidence_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1,
    })
}
