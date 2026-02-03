use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BamInvariantStatusV1 {
    Pass,
    Warn,
    Fail,
}

impl From<bijux_core::InvariantStatusV1> for BamInvariantStatusV1 {
    fn from(value: bijux_core::InvariantStatusV1) -> Self {
        match value {
            bijux_core::InvariantStatusV1::Pass => Self::Pass,
            bijux_core::InvariantStatusV1::Warn => Self::Warn,
            bijux_core::InvariantStatusV1::Fail => Self::Fail,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamStageVerdictV1 {
    pub stage_id: String,
    pub verdict: BamInvariantStatusV1,
    pub reasons: Vec<String>,
    pub key_metrics: serde_json::Value,
}

impl From<bijux_core::StageVerdictV1> for BamStageVerdictV1 {
    fn from(verdict: bijux_core::StageVerdictV1) -> Self {
        Self {
            stage_id: verdict.stage_id,
            verdict: verdict.verdict.into(),
            reasons: verdict.reasons,
            key_metrics: verdict.key_metrics,
        }
    }
}
