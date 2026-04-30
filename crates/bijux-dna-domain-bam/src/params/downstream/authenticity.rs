use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityEffectiveParams {
    pub mode: String,
    #[serde(default = "default_evidence_only")]
    pub evidence_only: bool,
    #[serde(default = "default_disallow_certification")]
    pub disallow_certification: bool,
}

fn default_evidence_only() -> bool {
    true
}

fn default_disallow_certification() -> bool {
    true
}
