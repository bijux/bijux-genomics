use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaxonomyRecordV1 {
    pub taxon_id: u64,
    pub taxon_name: String,
    pub rank: String,
    pub read_count: u64,
    #[serde(default)]
    pub fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassificationDbProvenanceV1 {
    pub db_name: String,
    pub db_version: String,
    pub db_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KrakenUniqRecordV1 {
    pub taxonomy: TaxonomyRecordV1,
    pub unique_kmer_count: u64,
    #[serde(default)]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrackenRecordV1 {
    pub taxonomy: TaxonomyRecordV1,
    pub estimated_reads: f64,
    #[serde(default)]
    pub estimated_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KrakenUniqClassificationMetricsV1 {
    pub schema_version: String,
    pub provenance: ClassificationDbProvenanceV1,
    pub taxonomy_table: Vec<KrakenUniqRecordV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrackenClassificationMetricsV1 {
    pub schema_version: String,
    pub provenance: ClassificationDbProvenanceV1,
    pub taxonomy_table: Vec<BrackenRecordV1>,
}
