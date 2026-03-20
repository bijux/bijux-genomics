use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::PairedMode;

pub const REMOVE_DUPLICATES_SCHEMA_VERSION: &str = "bijux.fastq.params.remove_duplicates.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DedupMode {
    Exact,
    SequenceIdentity,
    OpticalAware,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RemoveDuplicatesEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub dedup_mode: DedupMode,
    pub keep_order: bool,
}

impl RemoveDuplicatesEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "dedup_mode": self.dedup_mode,
            "keep_order": self.keep_order,
        })
    }
}
