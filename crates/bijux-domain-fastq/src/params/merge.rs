use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MergeEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub merge_overlap: Option<u32>,
    #[serde(default)]
    pub min_len: Option<u32>,
}

impl MergeEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "merge_policy": self.merge_overlap,
            "min_len": self.min_len,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
