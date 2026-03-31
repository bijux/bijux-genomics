use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::PairedMode;

pub const RRNA_DEPLETION_SCHEMA_VERSION: &str = "bijux.fastq.params.deplete_rrna.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RrnaEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub contaminant_db: Option<String>,
    pub database_artifact_id: String,
    #[serde(default)]
    pub database_build_id: Option<String>,
    pub screening_engine: RrnaScreeningEngine,
    pub report_format: RrnaReportFormat,
    #[serde(default)]
    pub emit_removed_reads: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RrnaScreeningEngine {
    Sortmerna,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RrnaReportFormat {
    SummaryTsvAndJson,
}

impl RrnaEffectiveParams {
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
        if self.database_artifact_id.trim().is_empty() {
            missing.push("database_artifact_id");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "contaminant_db": self.contaminant_db,
            "database_artifact_id": self.database_artifact_id,
            "database_build_id": self.database_build_id,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "screening_engine": self.screening_engine,
            "report_format": self.report_format,
            "emit_removed_reads": self.emit_removed_reads,
        })
    }
}
