use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::PairedMode;

pub const HOST_DEPLETION_SCHEMA_VERSION: &str = "bijux.fastq.params.deplete_host.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceScope {
    Host,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadRetentionPolicy {
    KeepNonHostReads,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MappingReportFormat {
    Bowtie2MetricsFile,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceMaskingPolicy {
    Unmasked,
    HardMasked,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceDecoyPolicy {
    None,
    Included,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HostDepletionEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub reference_scope: ReferenceScope,
    pub reference_catalog_id: String,
    pub reference_index_artifact_id: String,
    pub reference_index_backend: String,
    #[serde(default)]
    pub reference_build_id: Option<String>,
    #[serde(default)]
    pub reference_digest: Option<String>,
    pub masking_policy: ReferenceMaskingPolicy,
    pub decoy_policy: ReferenceDecoyPolicy,
    #[serde(default)]
    pub decoy_catalog_id: Option<String>,
    pub identity_threshold: f64,
    pub retained_read_policy: ReadRetentionPolicy,
    pub emit_removed_reads: bool,
    pub report_format: MappingReportFormat,
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
        if self.reference_index_artifact_id.trim().is_empty() {
            missing.push("reference_index_artifact_id");
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
            "reference_scope": self.reference_scope,
            "reference_catalog_id": self.reference_catalog_id,
            "reference_index_artifact_id": self.reference_index_artifact_id,
            "reference_index_backend": self.reference_index_backend,
            "reference_build_id": self.reference_build_id,
            "reference_digest": self.reference_digest,
            "masking_policy": self.masking_policy,
            "decoy_policy": self.decoy_policy,
            "decoy_catalog_id": self.decoy_catalog_id,
            "identity_threshold": self.identity_threshold,
            "retained_read_policy": self.retained_read_policy,
            "emit_removed_reads": self.emit_removed_reads,
            "report_format": self.report_format,
            "retain_unmapped_pairs": self.retain_unmapped_pairs,
        })
    }
}
