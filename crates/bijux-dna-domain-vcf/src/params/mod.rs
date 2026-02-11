use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PARAM_SCHEMA_V1: &str = "bijux.vcf.params.v1";

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum VcfEffectiveParams {
    Call(VcfCallParams),
    Filter(VcfFilterParams),
    Stats(VcfStatsParams),
}
