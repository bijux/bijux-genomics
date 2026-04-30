//! Alignment effective parameters and read-group policy.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::ReadGroupPolicy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadGroupSpec {
    pub id: String,
    pub sample: String,
    pub platform: String,
    pub library: String,
    #[serde(default)]
    pub platform_unit: Option<String>,
    #[serde(default)]
    pub lane_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
}

impl ReadGroupSpec {
    #[must_use]
    pub fn with_defaults(sample_id: &str) -> Self {
        Self {
            id: format!("{sample_id}.rg1"),
            sample: sample_id.to_string(),
            platform: "ILLUMINA".to_string(),
            library: "lib1".to_string(),
            platform_unit: Some(format!("{sample_id}.pu1")),
            lane_id: Some("L001".to_string()),
            run_id: None,
        }
    }

    #[must_use]
    pub fn library_id(&self) -> Option<String> {
        if self.library.trim().is_empty() {
            None
        } else {
            Some(self.library.clone())
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
    #[serde(default)]
    pub sensitivity_profile: Option<String>,
    #[serde(default)]
    pub seed_length: Option<u32>,
    pub build_indices: bool,
    pub emit_stats: bool,
}
