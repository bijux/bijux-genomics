use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const FILTER_LOW_COMPLEXITY_SCHEMA_VERSION: &str =
    "bijux.fastq.params.filter_low_complexity.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FilterLowComplexityEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub entropy_threshold: f64,
    #[serde(default)]
    pub polyx_threshold: Option<u32>,
}

impl FilterLowComplexityEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
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
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "entropy_threshold": self.entropy_threshold,
            "polyx_threshold": self.polyx_threshold,
        })
    }
}
