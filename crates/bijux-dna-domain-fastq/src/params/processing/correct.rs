use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::PairedMode;

pub const CORRECT_SCHEMA_VERSION: &str = "bijux.fastq.params.correct.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionEngine {
    Rcorrector,
    Musket,
    Lighter,
    Bayeshammer,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QualityEncoding {
    Phred33,
    Phred64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FastqCorrectParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub correction_engine: CorrectionEngine,
    pub quality_encoding: QualityEncoding,
    pub kmer_size: Option<u32>,
    pub max_memory_gb: Option<u32>,
    pub trusted_kmer_artifact: Option<PathBuf>,
    pub conservative_mode: bool,
}

impl FastqCorrectParams {
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
            "correction_engine": self.correction_engine,
            "quality_encoding": self.quality_encoding,
            "kmer_size": self.kmer_size,
            "max_memory_gb": self.max_memory_gb,
            "trusted_kmer_artifact": self.trusted_kmer_artifact,
            "conservative_mode": self.conservative_mode,
        })
    }
}
