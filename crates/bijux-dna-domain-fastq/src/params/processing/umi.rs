use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const UMI_SCHEMA_VERSION: &str = "bijux.fastq.params.umi.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UmiExtractionLocation {
    Read1Prefix,
    Read2Prefix,
    IndexRead,
    HeaderTag,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UmiReadNameTransform {
    AppendToHeader,
    ReplaceHeader,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UmiFailedExtractionPolicy {
    RefuseStage,
    RetainUnmodified,
    RouteToRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UmiDownstreamPropagation {
    HeaderOnly,
    HeaderAndReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqUmiParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub umi_pattern: Option<String>,
    pub extraction_location: UmiExtractionLocation,
    pub read_name_transform: UmiReadNameTransform,
    pub failed_extraction_policy: UmiFailedExtractionPolicy,
    pub downstream_propagation: UmiDownstreamPropagation,
}

impl FastqUmiParams {
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
            "umi_pattern": self.umi_pattern,
            "extraction_location": self.extraction_location,
            "read_name_transform": self.read_name_transform,
            "failed_extraction_policy": self.failed_extraction_policy,
            "downstream_propagation": self.downstream_propagation,
        })
    }
}
