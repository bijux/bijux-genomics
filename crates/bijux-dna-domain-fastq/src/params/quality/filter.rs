use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DamageMode, PairedMode};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FilterEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub max_n: Option<u32>,
    #[serde(default)]
    pub max_n_fraction: Option<f64>,
    #[serde(default)]
    pub max_n_count: Option<u32>,
    #[serde(default)]
    pub low_complexity_threshold: Option<f64>,
    #[serde(default)]
    pub entropy_threshold: Option<f64>,
    #[serde(default)]
    pub contaminant_db: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
    #[serde(default)]
    pub polyx_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_mode: Option<DamageMode>,
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
            "max_n_fraction": self.max_n_fraction,
            "max_n_count": self.max_n_count,
            "low_complexity_threshold": self.low_complexity_threshold,
            "entropy_threshold": self.entropy_threshold,
            "kmer_ref": self.contaminant_db,
            "n_policy": self.n_policy,
            "polyx_policy": self.polyx_policy,
            "damage_mode": self.damage_mode,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
