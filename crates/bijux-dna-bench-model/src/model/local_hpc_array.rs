//! Shared persisted local HPC array resource contracts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcArrayResources {
    pub cpus_per_task: u32,
    pub memory_mb: u32,
    pub time_limit: String,
    pub scratch_gb: u32,
}
