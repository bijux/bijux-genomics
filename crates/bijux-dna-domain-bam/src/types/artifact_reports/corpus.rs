//! Persisted BAM corpus reports and schema identities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BAM_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION: &str = "bijux.bam.bench_corpus_manifest.v1";

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BamBenchDatasetScenarioV1 {
    TinyAligned,
    UnmappedHeavy,
    DuplicateHeavy,
    LowCoverage,
    DamageLike,
    ReferenceMismatch,
    MissingIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamBenchCorpusDatasetManifestEntryV1 {
    pub dataset_id: String,
    pub scientific_scope: String,
    pub requires_index: bool,
    pub scenarios: Vec<BamBenchDatasetScenarioV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamBenchCorpusManifestV1 {
    pub schema_version: String,
    pub corpus_id: String,
    pub scenarios_covered: Vec<BamBenchDatasetScenarioV1>,
    pub ci_subset: Vec<String>,
    pub datasets: Vec<BamBenchCorpusDatasetManifestEntryV1>,
}
