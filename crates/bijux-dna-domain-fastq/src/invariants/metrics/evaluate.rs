use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::invariants::{InvariantStatusV1, StageVerdictV1};

use super::{merge, shared, stage_sets, trim_filter, validation};
use crate::invariants::evaluation::{worst_status, InvariantEvaluation, InvariantThresholds};
use crate::stages::ids::{
    STAGE_FILTER_READS, STAGE_MERGE_PAIRS, STAGE_TRIM_READS, STAGE_VALIDATE_READS,
};

pub use stage_sets::{CORE_STAGES, META_STAGES, OPTIONAL_STAGES};

#[must_use]
pub fn evaluate_invariants(
    stage_id: &StageId,
    metrics_json: &serde_json::Value,
    effective_params: &serde_json::Value,
    thresholds: &InvariantThresholds,
) -> InvariantEvaluation {
    let mut state = shared::initialize(stage_id, effective_params);

    if stage_id == &STAGE_TRIM_READS {
        trim_filter::evaluate_trim(&mut state, metrics_json, thresholds);
    } else if stage_id == &STAGE_FILTER_READS {
        trim_filter::evaluate_filter(&mut state, metrics_json, thresholds);
    } else if stage_id == &STAGE_VALIDATE_READS {
        validation::evaluate(&mut state, metrics_json);
    } else if stage_id == &STAGE_MERGE_PAIRS {
        merge::evaluate(&mut state, metrics_json, thresholds);
    }

    let mut verdict = InvariantStatusV1::Pass;
    let mut reasons = Vec::new();
    for entry in &state.results {
        verdict = worst_status(verdict, &entry.status);
        if entry.status != InvariantStatusV1::Pass {
            reasons.push(entry.id.clone());
        }
    }

    let stage_verdict = StageVerdictV1 {
        stage_id: stage_id.to_string(),
        verdict,
        reasons,
        key_metrics: serde_json::Value::Object(state.key_metrics),
    };

    InvariantEvaluation { results: state.results, verdict: stage_verdict }
}
