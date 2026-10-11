use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::super::EvidenceSourceIdentity;
use crate::evidence::tool_contract::{ContainerArtifactDelivery, ContainerProductSupport};

pub const EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1: &str =
    "bijux.container.external_artifact_admission.v1";
pub const EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2: &str =
    "bijux.container.external_artifact_admission.v2";
pub const EXTERNAL_ARTIFACT_IMAGE_SCHEME: &str = "external-artifact://";

#[must_use]
pub fn external_artifact_image_ref(tool_id: &str) -> String {
    format!("{EXTERNAL_ARTIFACT_IMAGE_SCHEME}{tool_id}")
}

#[must_use]
pub fn external_artifact_tool_id(image: &str) -> Option<&str> {
    image.strip_prefix(EXTERNAL_ARTIFACT_IMAGE_SCHEME).filter(|tool_id| !tool_id.is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExternalArtifactKind {
    Binary,
    ApptainerSif,
}

impl ExternalArtifactKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::ApptainerSif => "apptainer_sif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExternalArtifactAdmissionStatus {
    Admitted,
    BlockedExternalAccess,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExternalArtifactIdentity {
    pub kind: ExternalArtifactKind,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExternalArtifactVersionProbe {
    pub command: Vec<String>,
    pub output: String,
    pub output_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExternalArtifactAdmission {
    #[schemars(schema_with = "external_artifact_admission_schema_version")]
    pub schema_version: String,
    pub availability: ExternalArtifactAdmissionStatus,
    pub recorded_at_utc: String,
    pub repository_commit: String,
    pub catalog: EvidenceSourceIdentity,
    pub distributable_image_root: String,
    pub tool_id: String,
    pub version: String,
    pub product_support: ContainerProductSupport,
    pub artifact_delivery: ContainerArtifactDelivery,
    pub delivery_reason: String,
    pub stage_ids: Vec<String>,
    pub expected_binary: String,
    pub version_command: String,
    pub expected_version_regex: String,
    #[schemars(schema_with = "redistribution_prohibited")]
    pub redistribution_permitted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lawful_access_basis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ExternalArtifactIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_probe: Option<ExternalArtifactVersionProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker_reason: Option<String>,
}

fn external_artifact_admission_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2,
    })
}

fn redistribution_prohibited(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "boolean",
        "const": false,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExternalArtifactCommandBinding {
    pub tool_id: String,
    pub stage_id: String,
    pub artifact_sha256: String,
    pub executable: String,
    pub arguments: Vec<String>,
}
