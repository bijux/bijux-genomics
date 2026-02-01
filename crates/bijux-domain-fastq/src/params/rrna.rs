use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RrnaEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub contaminant_db: Option<String>,
}

impl RrnaEffectiveParams {
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
            "contaminant_db": self.contaminant_db,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
