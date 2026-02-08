use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreprocessEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub stages: Vec<String>,
    #[serde(default)]
    pub enable_contaminant_removal: bool,
}

impl PreprocessEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.stages.is_empty() {
            missing.push("stages");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "stages": self.stages,
            "enable_contaminant_removal": self.enable_contaminant_removal,
        })
    }
}
