use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FilterEffectiveParams {
    pub mapq_threshold: u8,
    pub include_flags: Vec<u16>,
    pub exclude_flags: Vec<u16>,
    pub min_length: u32,
    pub remove_duplicates: bool,
    pub base_quality_threshold: u8,
}
