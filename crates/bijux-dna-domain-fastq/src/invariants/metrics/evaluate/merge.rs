use crate::metrics::FastqMergeMetricsV1;

use super::shared::EvaluationState;
use crate::invariants::evaluation::{result, InvariantThresholds};

pub(super) fn evaluate(
    state: &mut EvaluationState,
    metrics_json: &serde_json::Value,
    thresholds: &InvariantThresholds,
) {
    if let Ok(metrics) = serde_json::from_value::<FastqMergeMetricsV1>(metrics_json.clone()) {
        state.key_metrics.insert(
            "merge_rate".to_string(),
            serde_json::Value::from(metrics.merge_rate),
        );
        let rate = metrics.merge_rate;
        if rate < thresholds.merge_rate_fail_low || rate > thresholds.merge_rate_fail_high {
            state.results.push(result(
                "merge_rate_range",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
                format!(
                    "merge_rate {:.3} outside fail range [{:.2}, {:.2}]",
                    rate, thresholds.merge_rate_fail_low, thresholds.merge_rate_fail_high
                ),
                Some("inspect merge parameters and read overlap suitability".to_string()),
            ));
        } else if rate < thresholds.merge_rate_warn_low || rate > thresholds.merge_rate_warn_high {
            state.results.push(result(
                "merge_rate_range",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Warn,
                format!(
                    "merge_rate {:.3} outside warn range [{:.2}, {:.2}]",
                    rate, thresholds.merge_rate_warn_low, thresholds.merge_rate_warn_high
                ),
                Some("inspect merge parameters and read overlap suitability".to_string()),
            ));
        } else {
            state.results.push(result(
                "merge_rate_range",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
                "merge_rate within expected range".to_string(),
                None,
            ));
        }
    } else {
        state.results.push(result(
            "metrics_parse",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
            "failed to parse merge metrics".to_string(),
            Some("verify metrics schema for fastq.merge_pairs".to_string()),
        ));
    }
}
