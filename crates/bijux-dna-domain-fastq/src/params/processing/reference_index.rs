use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const INDEX_REFERENCE_SCHEMA_VERSION: &str = "bijux.fastq.params.index_reference.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceIndexEffectiveParams {
    pub schema_version: String,
    pub threads: u32,
    pub index_format: String,
    pub output_artifact: String,
    pub report_artifact: String,
    pub index_prefix: Option<String>,
}

impl ReferenceIndexEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.index_format.trim().is_empty() {
            missing.push("index_format");
        }
        if self.output_artifact.trim().is_empty() {
            missing.push("output_artifact");
        }
        if self.report_artifact.trim().is_empty() {
            missing.push("report_artifact");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "threads": self.threads,
            "index_format": self.index_format,
            "output_artifact": self.output_artifact,
            "report_artifact": self.report_artifact,
            "index_prefix": self.index_prefix,
        })
    }
}
