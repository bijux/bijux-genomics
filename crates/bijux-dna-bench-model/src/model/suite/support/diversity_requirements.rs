use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiversityRequirements {
    pub min_dataset_count: usize,
    pub min_classes: usize,
    pub min_read_layouts: usize,
}
