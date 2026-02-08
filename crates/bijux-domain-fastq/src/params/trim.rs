use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub min_len: u32,
    #[serde(default)]
    pub q_cutoff: Option<u32>,
    pub adapter_policy: String,
    #[serde(default)]
    pub polyx_policy: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
    #[serde(default)]
    pub contaminant_policy: Option<String>,
}

impl TrimEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.adapter_policy.trim().is_empty() {
            missing.push("adapter_policy");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "min_len": self.min_len,
            "q": self.q_cutoff,
            "adapter_policy": self.adapter_policy,
            "polyx_policy": self.polyx_policy,
            "n_policy": self.n_policy,
            "contaminant_policy": self.contaminant_policy,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
