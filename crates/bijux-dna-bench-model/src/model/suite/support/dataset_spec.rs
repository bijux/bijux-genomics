//! Owner: bijux-dna-bench-model
//! Dataset contract for benchmark suites.

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
