use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const VALIDATE_SCHEMA_VERSION: &str = "bijux.fastq.params.validate_reads.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    Strict,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairSyncPolicy {
    NotApplicable,
    RequireHeaderSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub q_cutoff: Option<u32>,
    pub validation_mode: ValidationMode,
    pub pair_sync_policy: PairSyncPolicy,
}

impl ValidateEffectiveParams {
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
            "q_cutoff": self.q_cutoff,
            "validation_mode": self.validation_mode,
            "pair_sync_policy": self.pair_sync_policy,
        })
    }
}
