//! Persisted BAM resources reports and schema identities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BAM_RESOURCE_PLAN_SCHEMA_VERSION: &str = "bijux.bam.resource_plan.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamStageResourcePlanV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bytes: u64,
    pub cpu_threads: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
    pub scratch_gb: u32,
    pub walltime_minutes: u32,
    pub io_profile: String,
    pub input_origin: BamInputOriginV1,
    pub input_scale: BamInputScaleV1,
    pub requires_index: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamInputOriginV1 {
    Real,
    Synthetic,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BamInputScaleV1 {
    Tiny,
    Small,
    Medium,
    Large,
}
