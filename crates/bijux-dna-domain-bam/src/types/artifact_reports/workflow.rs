//! Persisted BAM workflow reports and schema identities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION: &str = "bijux.bam.advisory_boundary.v1";

pub const BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION: &str = "bijux.bam.workflow_template.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAdvisoryBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub advisory_only: bool,
    pub scientific_scope: String,
    #[serde(default)]
    pub evidence_inputs: Vec<String>,
    #[serde(default)]
    pub safe_for_claims: Vec<String>,
    #[serde(default)]
    pub unsafe_for_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamWorkflowModeV1 {
    Modern,
    AncientLike,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamWorkflowTemplateV1 {
    pub schema_version: String,
    pub template_id: String,
    pub mode: BamWorkflowModeV1,
    pub profile_id: String,
    pub summary: String,
    pub required_stages: Vec<String>,
    pub advisory_stages: Vec<String>,
}
