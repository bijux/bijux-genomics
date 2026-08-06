//! Owner: bijux-dna-bench-model
//! Typed source-acquisition and admitted-catalog contracts for real benchmark corpora.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{CORPUS_ACQUISITION_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CorpusAcquisitionLayout {
    Se,
    Pe,
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
    pub source_record: String,
    pub source_record_sha256: String,
    pub retrieved_at_utc: String,
    pub admitted_at_utc: String,
    pub access_classification: String,
    pub redistribution_classification: String,
    pub access_terms: String,
    pub scientific_name: String,
    pub samples: Vec<GovernedDataCatalogSample>,
    pub files: Vec<GovernedDataCatalogFile>,
    pub known_limitations: Vec<String>,
}

#[must_use]
pub fn corpus_acquisition_plan_schema() -> Schema {
    schemars::schema_for!(CorpusAcquisitionPlan)
}

#[must_use]
pub fn governed_data_catalog_schema() -> Schema {
    schemars::schema_for!(GovernedDataCatalog)
}

fn corpus_acquisition_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": CORPUS_ACQUISITION_SCHEMA_V1})
}

fn governed_data_catalog_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": GOVERNED_DATA_CATALOG_SCHEMA_V1})
}
