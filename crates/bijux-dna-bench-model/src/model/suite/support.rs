use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSpec {
    pub id: String,
    pub hash: String,
    pub size: u64,
    pub origin: String,
    pub class_label: String,
    pub read_layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicatePolicy {
    pub count: u32,
    pub warmup: u32,
    pub seeds: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiversityRequirements {
    pub min_dataset_count: usize,
    pub min_classes: usize,
    pub min_read_layouts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StratificationRequirement {
    pub key: String,
    pub required_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequirements {
    pub require_bootstrap: bool,
    pub require_outlier_detection: bool,
    pub min_replicates_for_bootstrap: u32,
}
