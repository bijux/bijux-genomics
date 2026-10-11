use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::qualification::{
    ArtifactAvailability, ArtifactDelivery, EvidenceVerdict, ProductSupport,
};

pub const CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1: &str = "bijux.container.release_evidence.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerReleaseEvidencePublication {
    #[schemars(schema_with = "release_evidence_schema_version")]
    pub schema_version: String,
    pub released_at_utc: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub qualification_sources_sha256: String,
    pub policy_sha256: String,
    pub tools: BTreeMap<String, ToolReleaseEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolReleaseEvidence {
    pub tool_id: String,
    pub product_support: ProductSupport,
    pub artifact_delivery: ArtifactDelivery,
    pub redistribution_permitted: bool,
    pub availability: ArtifactAvailability,
    pub qualification_evidence_set_sha256: String,
    pub verdict: EvidenceVerdict,
    #[serde(default)]
    pub reasons: Vec<String>,
}

fn release_evidence_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1,
    })
}
