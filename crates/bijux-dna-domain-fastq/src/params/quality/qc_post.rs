use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const REPORT_QC_SCHEMA_VERSION: &str = "bijux.fastq.params.report_qc.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QcAggregationEngine {
    Multiqc,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QcAggregationScope {
    FastqQcInputs,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QcPostEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub aggregation_engine: QcAggregationEngine,
    pub aggregation_scope: QcAggregationScope,
}

impl QcPostEffectiveParams {
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
            "aggregation_engine": self.aggregation_engine,
            "aggregation_scope": self.aggregation_scope,
        })
    }
}
