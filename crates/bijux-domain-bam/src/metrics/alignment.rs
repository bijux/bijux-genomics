use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlignmentCountsV1 {
    pub total: u64,
    pub primary: u64,
    pub mapped: u64,
    pub proper_pair: u64,
    pub duplicates: u64,
}

impl AlignmentCountsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            total: 0,
            primary: 0,
            mapped: 0,
            proper_pair: 0,
            duplicates: 0,
        }
    }
}
