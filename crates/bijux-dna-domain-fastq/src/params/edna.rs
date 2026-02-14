use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const EDNA_SCHEMA_VERSION: &str = "v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrimerNormalizationEffectiveParams {
    pub orientation_policy: String,
    pub primer_set_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChimeraDetectionEffectiveParams {
    pub method: String,
    pub chimera_removed_definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AsvInferenceEffectiveParams {
    pub requires_r_runtime: bool,
    pub output_table_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtuClusteringEffectiveParams {
    pub identity_threshold: f64,
    pub output_table_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AbundanceNormalizationEffectiveParams {
    pub method: String,
    pub expected_columns: Vec<String>,
}
