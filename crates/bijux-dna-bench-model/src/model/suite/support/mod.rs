mod dataset_spec;
mod diversity_requirements;
mod replicate_policy;
mod stratification_requirement;

pub use dataset_spec::DatasetSpec;
pub use diversity_requirements::DiversityRequirements;
pub use replicate_policy::ReplicatePolicy;
pub use stratification_requirement::StratificationRequirement;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequirements {
    pub require_bootstrap: bool,
    pub require_outlier_detection: bool,
    pub min_replicates_for_bootstrap: u32,
}
