use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequirements {
    pub require_bootstrap: bool,
    pub require_outlier_detection: bool,
    pub min_replicates_for_bootstrap: u32,
}
