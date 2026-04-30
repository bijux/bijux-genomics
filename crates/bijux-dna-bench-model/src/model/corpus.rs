//! Owner: bijux-dna-bench-model
//! Typed benchmark corpus contracts for deterministic fixture catalogs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CorpusDomain {
    Fastq,
    Bam,
    Vcf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CorpusScale {
    CiSmall,
    LocalMedium,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TruthSetStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TruthSetHook {
    pub status: TruthSetStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truth_set_id: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BackendComparisonSpec {
    pub comparison_id: String,
    pub stage_id: String,
    pub tools: Vec<String>,
    pub caveat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub domain: CorpusDomain,
    pub scale: CorpusScale,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scientific_caveats: Vec<String>,
    pub datasets: Vec<CorpusDatasetSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backend_comparisons: Vec<BackendComparisonSpec>,
}
