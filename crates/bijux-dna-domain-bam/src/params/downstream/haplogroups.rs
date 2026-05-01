use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HaplogroupEffectiveParams {
    pub reference_panel: String,
    #[serde(default = "default_reference_build")]
    pub reference_build: String,
    #[serde(default)]
    pub min_coverage: Option<f64>,
    #[serde(default)]
    pub population_scope: Option<String>,
    #[serde(default = "default_refuse_without_population_context")]
    pub refuse_without_population_context: bool,
}

fn default_reference_build() -> String {
    "unspecified".to_string()
}

fn default_refuse_without_population_context() -> bool {
    true
}
