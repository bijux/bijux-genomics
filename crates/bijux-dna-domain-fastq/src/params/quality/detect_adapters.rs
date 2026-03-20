use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const DETECT_ADAPTERS_SCHEMA_VERSION: &str = "bijux.fastq.params.detect_adapters.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterInspectionMode {
    EvidenceOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetectAdaptersEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub sample_reads: Option<u32>,
    pub inspection_mode: AdapterInspectionMode,
    pub report_only: bool,
    pub evidence_engine: String,
}

impl DetectAdaptersEffectiveParams {
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
        if self.evidence_engine.trim().is_empty() {
            missing.push("evidence_engine");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "sample_reads": self.sample_reads,
            "inspection_mode": self.inspection_mode,
            "report_only": self.report_only,
            "evidence_engine": self.evidence_engine,
        })
    }
}
