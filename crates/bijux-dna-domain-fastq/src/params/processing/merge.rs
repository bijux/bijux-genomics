use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const MERGE_SCHEMA_VERSION: &str = "bijux.fastq.params.merge_pairs.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MergeEngine {
    Pear,
    Vsearch,
    Bbmerge,
    Flash2,
    Leehom,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnmergedReadPolicy {
    EmitUnmergedPairs,
    OmitUnmergedPairs,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MergeEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub merge_overlap: Option<u32>,
    #[serde(default)]
    pub min_len: Option<u32>,
    pub merge_engine: MergeEngine,
    pub unmerged_read_policy: UnmergedReadPolicy,
}

impl MergeEffectiveParams {
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
            "merge_policy": self.merge_overlap,
            "min_len": self.min_len,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "merge_engine": self.merge_engine,
            "unmerged_read_policy": self.unmerged_read_policy,
        })
    }
}
