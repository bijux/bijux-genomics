//! Canonical persisted normalized result records grouped by benchmark domain.

pub mod bam;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDownstreamProbeRecord {
    pub schema_version: String,
    pub stage_id: String,
    pub input_contract: String,
    pub complete: bool,
}
