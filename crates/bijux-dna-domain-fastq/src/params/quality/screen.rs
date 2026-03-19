use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const HOST_DEPLETION_SCHEMA_VERSION: &str = "bijux.fastq.params.deplete_host.v1";
pub const REFERENCE_DEPLETION_SCHEMA_VERSION: &str =
    "bijux.fastq.params.deplete_reference_contaminants.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScreenEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub contaminant_db: Option<String>,
}

impl ScreenEffectiveParams {
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HostDepletionEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub host_reference: String,
    pub index_artifact: String,
    pub retain_unmapped_pairs: bool,
}

impl HostDepletionEffectiveParams {
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
        if self.host_reference.trim().is_empty() {
            missing.push("host_reference");
        }
        if self.index_artifact.trim().is_empty() {
            missing.push("index_artifact");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "host_reference": self.host_reference,
            "index_artifact": self.index_artifact,
            "retain_unmapped_pairs": self.retain_unmapped_pairs,
        })
    }
}

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
