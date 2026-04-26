use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PARAM_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct VcfCallParams {
    pub schema_version: String,
    pub caller: String,
    pub sample_name: String,
    pub reference_fasta: Option<String>,
    pub min_base_quality: u8,
    pub min_mapping_quality: u8,
}

impl Default for VcfCallParams {
    fn default() -> Self {
        Self {
            schema_version: PARAM_SCHEMA_V1.to_string(),
            caller: "bcftools".to_string(),
            sample_name: "sample".to_string(),
            reference_fasta: None,
            min_base_quality: 20,
            min_mapping_quality: 20,
        }
    }
}
