use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;

pub const HOST_DEPLETION_SCHEMA_VERSION: &str = "bijux.fastq.params.deplete_host.v1";
pub const REFERENCE_DEPLETION_SCHEMA_VERSION: &str =
    "bijux.fastq.params.deplete_reference_contaminants.v1";
pub const RRNA_DEPLETION_SCHEMA_VERSION: &str = "bijux.fastq.params.deplete_rrna.v1";
mod taxonomy;

pub use taxonomy::{
    ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
    TaxonomyReportFormat, SCREEN_TAXONOMY_SCHEMA_VERSION,
};

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
