use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PARAM_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct VcfFilterParams {
    pub schema_version: String,
    pub sample_name: String,
    pub min_qual: f64,
    pub require_pass: bool,
    pub normalize: bool,
    pub require_bgzip_tabix: bool,
    pub production_profile: bool,
}

impl Default for VcfFilterParams {
    fn default() -> Self {
        Self {
            schema_version: PARAM_SCHEMA_V1.to_string(),
            sample_name: "sample".to_string(),
            min_qual: 30.0,
            require_pass: true,
            normalize: true,
            require_bgzip_tabix: true,
            production_profile: false,
        }
    }
}
