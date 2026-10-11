//! Owner: bijux-dna-bench-model
//! Typed benchmark corpus contracts for deterministic fixture catalogs.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::contract::{
    BENCHMARK_DATA_INDEX_SCHEMA_V1, BUNDLE_SCHEMA_V1, CORPUS_SCHEMA_V1,
    FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1,
};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEligibility {
    Eligible,
    Incomplete,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataIntegrityStatus {
    Verified,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataIndexFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub role: String,
    pub legacy_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkTemporalProvenance {
    pub field: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataIndexRecord {
    pub id: String,
    pub catalog_path: String,
    pub catalog_sha256: String,
    pub purpose: String,
    pub source_identities: Vec<String>,
    pub access_classification: String,
    pub redistribution_classification: String,
    pub scientific_name: Option<String>,
    pub reference_assembly: Option<String>,
    pub temporal_provenance: BenchmarkTemporalProvenance,
    pub files: Vec<BenchmarkDataIndexFile>,
    pub known_limitations: Vec<String>,
    pub integrity_status: DataIntegrityStatus,
    pub benchmark_eligibility: BenchmarkEligibility,
    pub missing_provenance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataIndexSummary {
    pub record_count: u64,
    pub file_count: u64,
    pub size_bytes: u64,
    pub integrity_verified_count: u64,
    pub benchmark_eligibility: BTreeMap<BenchmarkEligibility, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataIndex {
    #[schemars(schema_with = "benchmark_data_index_schema_version")]
    pub schema_version: String,
    pub created_at_utc: String,
    pub data_root: String,
    pub summary: BenchmarkDataIndexSummary,
    pub records: Vec<BenchmarkDataIndexRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CorpusDomain {
    Fastq,
    Bam,
    Vcf,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CorpusScale {
    CiSmall,
    LocalMedium,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TruthSetStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TruthSetHook {
    pub status: TruthSetStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truth_set_id: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BackendComparisonSpec {
    pub comparison_id: String,
    pub stage_id: String,
    pub tools: Vec<String>,
    pub caveat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DriftScenarioSpec {
    pub scenario_id: String,
    pub drift_axis: String,
    pub baseline_label: String,
    pub candidate_label: String,
    pub caveat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusDatasetSpec {
    pub dataset_id: String,
    pub fixture: String,
    pub read_layout: String,
    pub class_label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub case_tags: Vec<String>,
    pub truth_set: TruthSetHook,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusManifest {
    #[schemars(schema_with = "benchmark_corpus_schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub domain: CorpusDomain,
    pub scale: CorpusScale,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scientific_caveats: Vec<String>,
    pub datasets: Vec<CorpusDatasetSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backend_comparisons: Vec<BackendComparisonSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drift_scenarios: Vec<DriftScenarioSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkBundleManifest {
    #[schemars(schema_with = "benchmark_bundle_schema_version")]
    pub schema_version: String,
    pub bundle_id: String,
    pub corpora: Vec<String>,
    pub environment_label: String,
    pub metrics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scientific_caveats: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkFastqLayout {
    SingleEnd,
    PairedEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmittedFastqCorpusFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mate: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmittedFastqCorpusSample {
    pub sample_id: String,
    pub input_layout: BenchmarkFastqLayout,
    pub files: Vec<AdmittedFastqCorpusFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmittedFastqCorpus {
    pub corpus_id: String,
    pub data_index_path: String,
    pub data_index_sha256: String,
    pub catalog_path: String,
    pub catalog_sha256: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub input_bytes: u64,
    pub samples: Vec<AdmittedFastqCorpusSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmittedBamCorpusFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmittedBamCorpus {
    pub corpus_id: String,
    pub sample_id: String,
    pub data_index_path: String,
    pub data_index_sha256: String,
    pub catalog_path: String,
    pub catalog_sha256: String,
    pub bam: AdmittedBamCorpusFile,
    pub bam_index: AdmittedBamCorpusFile,
    pub input_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FastqReadLengthExpectationSample {
    pub sample_id: String,
    pub input_layout: BenchmarkFastqLayout,
    pub file_count: u64,
    pub read_count: u64,
    pub minimum_read_length: u64,
    pub mean_read_length: f64,
    pub median_read_length: f64,
    pub maximum_read_length: u64,
    pub distinct_read_lengths: u64,
    pub summary_path: String,
    pub summary_sha256: String,
    pub histogram_path: String,
    pub histogram_sha256: String,
    pub exact_rows_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FastqReadLengthExpectations {
    pub schema_version: String,
    pub corpus_id: String,
    pub derivation_sha256: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub data_index_sha256: String,
    pub catalog_sha256: String,
    pub samples: Vec<FastqReadLengthExpectationSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FastqProfileReadsExpectationsProvenance {
    #[serde(deserialize_with = "deserialize_profile_reads_expectations_schema_version")]
    #[schemars(schema_with = "profile_reads_expectations_schema_version")]
    pub schema_version: String,
    pub corpus_id: String,
    pub sample_count: u64,
    pub file_count: u64,
    pub data_index_sha256: String,
    pub catalog_sha256: String,
    pub read_length_derivation_sha256: String,
    pub expected_profile_sha256: String,
}

#[must_use]
pub fn benchmark_corpus_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkCorpusManifest)
}

#[must_use]
pub fn benchmark_bundle_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkBundleManifest)
}

#[must_use]
pub fn benchmark_data_index_schema() -> Schema {
    schemars::schema_for!(BenchmarkDataIndex)
}

#[must_use]
pub fn fastq_profile_reads_expectations_schema() -> Schema {
    schemars::schema_for!(FastqProfileReadsExpectationsProvenance)
}

fn benchmark_data_index_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_DATA_INDEX_SCHEMA_V1})
}

fn benchmark_corpus_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": CORPUS_SCHEMA_V1})
}

fn benchmark_bundle_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BUNDLE_SCHEMA_V1})
}

fn profile_reads_expectations_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1})
}

fn deserialize_profile_reads_expectations_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported FASTQ profile expectation: {value}")))
    }
}
