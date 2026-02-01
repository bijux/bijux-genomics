use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FilterEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub max_n: Option<u32>,
    #[serde(default)]
    pub low_complexity_threshold: Option<f64>,
    #[serde(default)]
    pub contaminant_db: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
}

impl FilterEffectiveParams {
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
            "max_n": self.max_n,
            "low_complexity_threshold": self.low_complexity_threshold,
            "kmer_ref": self.contaminant_db,
            "n_policy": self.n_policy,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
