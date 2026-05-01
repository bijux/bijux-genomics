use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KinshipEffectiveParams {
    pub reference_panel: String,
    #[serde(default = "default_reference_build")]
    pub reference_build: String,
    #[serde(default = "default_population_scope")]
    pub population_scope: String,
    pub min_overlap_snps: u32,
    #[serde(default = "default_requires_cohort_context")]
    pub requires_cohort_context: bool,
}

fn default_reference_build() -> String {
    "unspecified".to_string()
}

fn default_population_scope() -> String {
    "unspecified".to_string()
}

fn default_requires_cohort_context() -> bool {
    true
}
