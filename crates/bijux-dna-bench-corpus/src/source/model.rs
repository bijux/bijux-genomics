use super::{BenchmarkCorpusClass, FASTQ_CORPUS_SOURCE_SCHEMA_V1};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

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

#[must_use]
pub fn fastq_corpus_source_spec_schema() -> Schema {
    schemars::schema_for!(FastqCorpusSourceSpec)
}
fn fastq_corpus_source_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":FASTQ_CORPUS_SOURCE_SCHEMA_V1})
}
