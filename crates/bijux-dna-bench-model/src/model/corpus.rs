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
#[serde(deny_unknown_fields)]
pub struct CorpusDatasetSpec {
    pub dataset_id: String,
    pub fixture: String,
    pub read_layout: String,
    pub class_label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub case_tags: Vec<String>,
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
}
