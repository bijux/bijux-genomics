use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const SECURITY_PROBE_PUBLICATION_SCHEMA_V1: &str =
    "bijux.container.security_probe_publication.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecurityProbeStatus {
    Collected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityProbeArtifact {
    pub uri: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityStructuralReport {
    pub suid_sgid_paths: Vec<String>,
    pub world_writable_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityRuntimeProbe {
    pub exit_code: i32,
    pub stdout: SecurityProbeArtifact,
    pub stderr: SecurityProbeArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityRuntimeReport {
    pub network_isolated: SecurityRuntimeProbe,
    pub non_root_uid: SecurityRuntimeProbe,
    pub system_read_only: SecurityRuntimeProbe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolSecurityProbe {
    pub tool_id: String,
    pub version: String,
    pub image_sha256: String,
    pub tool_contract_sha256: String,
    pub build_evidence_id: String,
    pub status: SecurityProbeStatus,
    pub sbom: Option<SecurityProbeArtifact>,
    pub vulnerability_report: Option<SecurityProbeArtifact>,
    pub secret_report: Option<SecurityProbeArtifact>,
    pub signature_stdout: Option<SecurityProbeArtifact>,
    pub signature_stderr: Option<SecurityProbeArtifact>,
    pub signature_verified: bool,
    pub structural_report: Option<SecurityProbeArtifact>,
    pub runtime_report: Option<SecurityProbeArtifact>,
    pub raw_manifest: SecurityProbeArtifact,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityProbePublication {
    #[schemars(schema_with = "security_probe_publication_schema_version")]
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub inventory_sha256: String,
    pub scanner_completion_sha256: String,
    pub database_completion_sha256: String,
    pub tools: BTreeMap<String, ToolSecurityProbe>,
}

fn security_probe_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
    })
}
