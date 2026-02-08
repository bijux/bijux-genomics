use bijux_dna_core::prelude::invariants::{InvariantStatusV1, StageVerdictV1};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BamInvariantStatusV1 {
    Pass,
    Warn,
    Fail,
}

impl From<InvariantStatusV1> for BamInvariantStatusV1 {
    fn from(value: InvariantStatusV1) -> Self {
        match value {
            InvariantStatusV1::Pass => Self::Pass,
            InvariantStatusV1::Warn => Self::Warn,
            InvariantStatusV1::Fail => Self::Fail,
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

impl From<StageVerdictV1> for BamStageVerdictV1 {
    fn from(verdict: StageVerdictV1) -> Self {
        Self {
            stage_id: verdict.stage_id,
            verdict: verdict.verdict.into(),
            reasons: verdict.reasons,
            key_metrics: verdict.key_metrics,
        }
    }
}
