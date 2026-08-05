use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::EvidenceSourceIdentity;

pub const IMAGE_ACTIVATION_PLAN_SCHEMA_V1: &str = "bijux.container.image_activation_plan.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageActivationPlan {
    pub schema_version: String,
    pub state: ImageActivationPlanState,
    pub created_at_utc: String,
    pub inventory: EvidenceSourceIdentity,
    pub source_root: String,
    pub active_root: String,
    pub file_count: u64,
    pub size_bytes: u64,
    pub publications: Vec<ImageActivationPublication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolled_back_at_utc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageActivationPlanState {
    Previewed,
    Applying,
    Applied,
    RollingBack,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageActivationPublication {
    pub tool: String,
    pub source: String,
    pub target: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_at_utc: Option<String>,
}
