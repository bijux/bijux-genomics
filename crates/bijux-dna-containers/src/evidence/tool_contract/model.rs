use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::catalog::{ContainerArtifactDelivery, ContainerProductSupport};

pub const CONTAINER_TOOL_CONTRACT_SCHEMA_V2: &str = "bijux.container.tool_contract.v2";
pub const CONTAINER_TOOL_CONTRACT_SCHEMA_V3: &str = "bijux.container.tool_contract.v3";
pub const TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V2: &str =
    "bijux.container.tool_image_identity_evidence.v2";
pub const TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V3: &str =
    "bijux.container.tool_image_identity_evidence.v3";
pub const CONTAINER_TOOL_CONTRACT_PATH: &str = "/usr/share/bijux/tool-contract.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerToolContract {
    #[schemars(schema_with = "tool_contract_schema_version")]
    pub schema_version: String,
    pub tool_id: String,
    pub version: String,
    pub product_support: ContainerProductSupport,
    pub artifact_delivery: ContainerArtifactDelivery,
    pub redistribution_permitted: bool,
    pub catalog_entry_sha256: String,
    pub catalog_sha256: String,
    pub recipe_path: String,
    pub recipe_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_commit: Option<String>,
    pub repository_commit: String,
    pub architecture: String,
    pub build_evidence_id: String,
    pub expected_binary: String,
    pub version_command: String,
    pub expected_version_regex: String,
    pub interface_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolImageIdentityEvidence {
    #[schemars(schema_with = "tool_image_identity_evidence_schema_version")]
    pub schema_version: String,
    pub image_sha256: String,
    pub contract: ContainerToolContract,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolContractBuildRequest {
    pub repository_root: PathBuf,
    pub tool_id: String,
    pub repository_commit: String,
    pub architecture: String,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedToolContractBuild {
    pub contract: ContainerToolContract,
    pub contract_path: PathBuf,
    pub rendered_definition_path: PathBuf,
}

fn tool_contract_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_TOOL_CONTRACT_SCHEMA_V3,
    })
}

fn tool_image_identity_evidence_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V3,
    })
}
