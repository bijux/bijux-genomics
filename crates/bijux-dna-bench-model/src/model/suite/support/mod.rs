mod dataset_spec;
mod replicate_policy;

pub use dataset_spec::DatasetSpec;
pub use replicate_policy::ReplicatePolicy;

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
