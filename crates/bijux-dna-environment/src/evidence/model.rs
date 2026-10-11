use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1: &str =
    "bijux.container.image_inventory_publication.v1";
pub const CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1: &str =
    "bijux.container.runtime_verification_publication.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContainerImageInventoryPublication {
    #[schemars(schema_with = "image_inventory_publication_schema_version")]
    pub schema_version: String,
    pub catalog_sha256: String,
    pub repository_commit: String,
    pub publication: ImageInventoryPublicationIdentity,
    pub tools: Vec<ContainerImageInventoryTool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImageInventoryPublicationIdentity {
    pub raw_schema_version: String,
    pub raw_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContainerImageInventoryTool {
    pub tool: String,
    pub version: String,
    pub support_disposition: ContainerSupportDisposition,
    pub apptainer_supported: bool,
    pub state: ContainerImageState,
    pub selected_image_sha256: Option<String>,
    #[serde(default)]
    pub candidates: Vec<ContainerImageCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContainerSupportDisposition {
    Retained,
    Unsupported,
    Retired,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContainerImageState {
    Current,
    Historical,
    Missing,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContainerImageCandidate {
    pub admitted: bool,
    pub current_recipe: bool,
    pub disposition: ImageCandidateDisposition,
    pub path: String,
    pub sha256: String,
    #[serde(default)]
    pub build_evidence: Vec<ContainerBuildEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageCandidateDisposition {
    Canonical,
    HistoricalRunDependency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContainerBuildEvidence {
    pub status: BuildEvidenceStatus,
    pub definition_sha256: String,
    pub repository_commit: String,
    pub build_context: Option<Vec<BuildContextIdentity>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildEvidenceStatus {
    Built,
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BuildContextIdentity {
    pub path: String,
    pub git_object: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeVerificationPublication {
    #[schemars(schema_with = "runtime_verification_publication_schema_version")]
    pub schema_version: String,
    pub catalog: EvidenceSourceIdentity,
    pub inventory: EvidenceSourceIdentity,
    pub repository_commit: String,
    pub tools: BTreeMap<String, ContainerRuntimeEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceSourceIdentity {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContainerRuntimeEvidence {
    pub version: String,
    pub image_sha256: String,
    pub status: RuntimeEvidenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEvidenceStatus {
    Passed,
    Failed,
    Blocked,
}

fn image_inventory_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    })
}

fn runtime_verification_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
    })
}
