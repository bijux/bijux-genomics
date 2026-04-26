use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PARAM_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct VcfStatsParams {
    pub schema_version: String,
    pub sample_name: String,
    pub compute_titv: bool,
    pub collect_depth_distribution: bool,
}

impl Default for VcfStatsParams {
    fn default() -> Self {
        Self {
            schema_version: PARAM_SCHEMA_V1.to_string(),
            sample_name: "sample".to_string(),
            compute_titv: true,
            collect_depth_distribution: true,
        }
    }
}
