use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub requires_phased: bool,
    pub supports_gl_input: bool,
    pub supports_minimac_m3vcf: bool,
    pub glimpse_reference_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub coordinate_system: String,
}
