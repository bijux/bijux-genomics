use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1};

use crate::invariants::evaluation::result;
use crate::parse_effective_params;
use crate::EffectiveParams;

pub(super) struct EvaluationState {
    pub results: Vec<InvariantResultV1>,
    pub key_metrics: serde_json::Map<String, serde_json::Value>,
    pub parsed_params: Option<EffectiveParams>,
}

#[must_use]
pub(super) fn initialize(
    stage_id: &StageId,
    effective_params: &serde_json::Value,
) -> EvaluationState {
    let parsed_params = parse_effective_params(stage_id, effective_params);
    let mut results = Vec::new();

    if let Some(params) = parsed_params.as_ref() {
        let missing = params.missing_required_fields();
        if missing.is_empty() {
            results.push(result(
                "effective_params_present",
                InvariantStatusV1::Pass,
                "effective params present with required fields".to_string(),
                None,
            ));
        } else {
            results.push(result(
                "effective_params_present",
                InvariantStatusV1::Fail,
                format!("missing effective params fields: {}", missing.join(", ")),
                Some("populate canonical effective params for this stage".to_string()),
            ));
        }
    } else {
        results.push(result(
            "effective_params_present",
            InvariantStatusV1::Fail,
            "effective params missing or malformed".to_string(),
            Some("emit canonical effective params JSON in stage planner".to_string()),
        ));
    }

    EvaluationState { results, key_metrics: serde_json::Map::new(), parsed_params }
}
