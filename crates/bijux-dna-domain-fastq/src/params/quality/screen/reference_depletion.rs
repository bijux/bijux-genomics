use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::PairedMode;

pub const REFERENCE_DEPLETION_SCHEMA_VERSION: &str =
    "bijux.fastq.params.deplete_reference_contaminants.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceContaminantEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub reference_catalog_id: String,
    pub contaminant_reference: String,
    pub index_artifact: String,
    pub reference_index_backend: String,
    #[serde(default)]
    pub reference_build_id: Option<String>,
    #[serde(default)]
    pub reference_digest: Option<String>,
    pub retain_unmapped_pairs: bool,
}

impl ReferenceContaminantEffectiveParams {
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
        if self.contaminant_reference.trim().is_empty() {
            missing.push("contaminant_reference");
        }
        if self.index_artifact.trim().is_empty() {
            missing.push("index_artifact");
        }
        if self.reference_catalog_id.trim().is_empty() {
            missing.push("reference_catalog_id");
        }
        if self.reference_index_backend.trim().is_empty() {
            missing.push("reference_index_backend");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "reference_catalog_id": self.reference_catalog_id,
            "contaminant_reference": self.contaminant_reference,
            "index_artifact": self.index_artifact,
            "reference_index_backend": self.reference_index_backend,
            "reference_build_id": self.reference_build_id,
            "reference_digest": self.reference_digest,
            "retain_unmapped_pairs": self.retain_unmapped_pairs,
        })
    }
}
