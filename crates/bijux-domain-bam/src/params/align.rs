//! Alignment effective parameters and read-group policy.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::sample_meta::ReadGroupPolicy;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadGroupSpec {
    pub id: String,
    pub sample: String,
    pub platform: String,
    pub library: String,
}

impl ReadGroupSpec {
    #[must_use]
    pub fn with_defaults(sample_id: &str) -> Self {
        Self {
            id: format!("{sample_id}.rg1"),
            sample: sample_id.to_string(),
            platform: "ILLUMINA".to_string(),
            library: "lib1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlignEffectiveParams {
    pub aligner: String,
    pub preset: String,
    pub threads: u32,
    pub reference: String,
    pub reference_digest: String,
    pub rg_policy: ReadGroupPolicy,
    pub read_group: ReadGroupSpec,
    pub build_indices: bool,
    pub emit_stats: bool,
}
