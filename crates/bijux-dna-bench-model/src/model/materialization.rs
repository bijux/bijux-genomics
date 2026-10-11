//! Owner: bijux-dna-bench-model
//! Typed source-acquisition and admitted-catalog contracts for real benchmark corpora.

use std::path::PathBuf;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::contract::{
    BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1, CORPUS_ACQUISITION_SCHEMA_V2,
    FASTQ_CORPUS_SOURCE_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};
use crate::model::BenchmarkCorpusClass;
use crate::model::GovernedDerivedDataCatalog;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CorpusAcquisitionLayout {
    Se,
    Pe,
}

impl CorpusAcquisitionLayout {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Se => "se",
            Self::Pe => "pe",
        }
    }
}

impl std::fmt::Display for CorpusAcquisitionLayout {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusSourcePolicy {
    pub class: BenchmarkCorpusClass,
    pub access_classification: String,
    pub redistribution_classification: String,
    pub access_terms_url: String,
    pub attribution: String,
    pub projected_storage_bytes: u64,
    pub projected_cpu_hours: u64,
    pub projection_basis: String,
    pub truth_roles: Vec<String>,
    pub truth_strategy: String,
    pub known_limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusEvaluationDesign {
    pub sample_context: String,
    pub assay_context: String,
    pub strata: Vec<String>,
    pub controls: Vec<String>,
    pub held_out_split: String,
    pub lineage: String,
    pub allowed_uses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FastqCorpusSourceSample {
    pub accession: String,
    pub study_accession: String,
    pub era: String,
    pub layout: CorpusAcquisitionLayout,
    pub size_band: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FastqCorpusSourceSpec {
    #[schemars(schema_with = "fastq_corpus_source_schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub portfolio_id: String,
    #[serde(flatten)]
    pub policy: CorpusSourcePolicy,
    #[serde(flatten)]
    pub evaluation: CorpusEvaluationDesign,
    pub species: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub species_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_root: Option<String>,
    pub description: String,
    pub source_authority: String,
    pub target_total: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ancient_se: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ancient_pe: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_modern_se: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_modern_pe: Option<usize>,
    pub samples: Vec<FastqCorpusSourceSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusAcquisitionFile {
    pub mate: Option<u8>,
    pub name: String,
    pub source_url: String,
    pub source_md5: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusAcquisitionSample {
    pub accession: String,
    pub study_accession: String,
    pub sample_accession: String,
    pub experiment_accession: String,
    pub era: String,
    pub layout: CorpusAcquisitionLayout,
    pub size_band: String,
    pub selection_reason: String,
    pub scientific_name: String,
    pub library_source: String,
    pub library_strategy: String,
    pub library_selection: String,
    pub instrument_platform: String,
    pub instrument_model: String,
    pub archive_base_count: u64,
    pub archive_read_count: u64,
    pub first_public: String,
    pub last_updated: String,
    pub source_report_url: String,
    pub files: Vec<CorpusAcquisitionFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusAcquisitionPlan {
    #[schemars(schema_with = "corpus_acquisition_schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub portfolio_id: String,
    pub species: String,
    pub retrieved_at_utc: String,
    pub source_authority: String,
    pub description: String,
    #[serde(flatten)]
    pub policy: CorpusSourcePolicy,
    #[serde(flatten)]
    pub evaluation: CorpusEvaluationDesign,
    pub source_spec: String,
    pub source_spec_sha256: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub total_size_bytes: u64,
    pub samples: Vec<CorpusAcquisitionSample>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorpusMaterializationDisposition {
    Downloaded,
    VerifiedExisting,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogSample {
    pub accession: String,
    pub study_accession: String,
    pub sample_accession: String,
    pub experiment_accession: String,
    pub era: String,
    pub layout: CorpusAcquisitionLayout,
    pub size_band: String,
    pub selection_reason: String,
    pub scientific_name: String,
    pub library_source: String,
    pub library_strategy: String,
    pub library_selection: String,
    pub instrument_platform: String,
    pub instrument_model: String,
    pub archive_base_count: u64,
    pub archive_read_count: u64,
    pub first_public: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub kind: String,
    pub accession: String,
    pub study_accession: String,
    pub sample_accession: String,
    pub experiment_accession: String,
    pub era: String,
    pub layout: CorpusAcquisitionLayout,
    pub mate: Option<u8>,
    pub read_count: u64,
    pub base_count: u64,
    pub source_url: String,
    pub source_md5: String,
    pub materialization_status: CorpusMaterializationDisposition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalog {
    #[schemars(schema_with = "governed_data_catalog_schema_version")]
    pub schema_version: String,
    pub id: String,
    pub purpose: String,
    pub source: String,
    #[serde(flatten)]
    pub policy: CorpusSourcePolicy,
    #[serde(flatten)]
    pub evaluation: CorpusEvaluationDesign,
    pub source_record: String,
    pub source_record_sha256: String,
    pub retrieved_at_utc: String,
    pub admitted_at_utc: String,
    pub scientific_name: String,
    pub samples: Vec<GovernedDataCatalogSample>,
    pub files: Vec<GovernedDataCatalogFile>,
}

/// Source-addressed preservation of a governed catalog whose domain-specific shape predates v2.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedPreservedDataCatalog {
    #[schemars(schema_with = "governed_data_catalog_schema_version")]
    pub schema_version: String,
    pub id: String,
    pub source_catalog_path: String,
    pub source_catalog_sha256: String,
    pub source_catalog_size_bytes: u64,
    pub source_catalog_schema_version: String,
    pub preserved_at_utc: String,
    pub preserved_catalog: serde_json::Value,
}

/// A v2 catalog is either native typed materialization or a digest-bound historical authority.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum GovernedDataCatalogDocument {
    Materialized(Box<GovernedDataCatalog>),
    Derived(Box<GovernedDerivedDataCatalog>),
    Preserved(GovernedPreservedDataCatalog),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusMaterializationPublication {
    #[serde(deserialize_with = "deserialize_materialization_publication_schema_version")]
    #[schemars(schema_with = "materialization_publication_schema_version")]
    pub schema_version: String,
    pub catalog_path: PathBuf,
    pub catalog_sha256: String,
    pub downloaded_files: u64,
    pub verified_existing_files: u64,
    pub catalog: GovernedDataCatalog,
}

#[must_use]
pub fn corpus_acquisition_plan_schema() -> Schema {
    schemars::schema_for!(CorpusAcquisitionPlan)
}

#[must_use]
pub fn fastq_corpus_source_spec_schema() -> Schema {
    schemars::schema_for!(FastqCorpusSourceSpec)
}

#[must_use]
pub fn governed_data_catalog_schema() -> Schema {
    schemars::schema_for!(GovernedDataCatalogDocument)
}

#[must_use]
pub fn corpus_materialization_publication_schema() -> Schema {
    schemars::schema_for!(CorpusMaterializationPublication)
}

fn corpus_acquisition_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": CORPUS_ACQUISITION_SCHEMA_V2})
}

fn fastq_corpus_source_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": FASTQ_CORPUS_SOURCE_SCHEMA_V1})
}

fn governed_data_catalog_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": GOVERNED_DATA_CATALOG_SCHEMA_V2})
}

fn materialization_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1
    })
}

fn deserialize_materialization_publication_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported corpus materialization publication: {value}")))
    }
}
