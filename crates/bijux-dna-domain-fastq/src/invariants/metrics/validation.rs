use crate::metrics::FastqValidateMetricsV1;

use super::shared::EvaluationState;
use crate::invariants::evaluation::result;

pub(super) fn evaluate(state: &mut EvaluationState, metrics_json: &serde_json::Value) {
    if let Ok(metrics) = serde_json::from_value::<FastqValidateMetricsV1>(metrics_json.clone()) {
        state
            .key_metrics
            .insert("reads_invalid".to_string(), serde_json::Value::from(metrics.reads_invalid));
        state.key_metrics.insert(
            "strict_pass".to_string(),
            serde_json::to_value(metrics.strict_pass).unwrap_or(serde_json::Value::Null),
        );
        state.key_metrics.insert(
            "failure_class".to_string(),
            serde_json::to_value(metrics.failure_class.clone()).unwrap_or(serde_json::Value::Null),
        );

        if metrics.reads_invalid > 0 {
            state.results.push(result(
                "validate_malformed_reads",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
                format!("reads_invalid {} > 0", metrics.reads_invalid),
                Some("inspect input FASTQ integrity and re-run validation".to_string()),
            ));
        } else {
            state.results.push(result(
                "validate_malformed_reads",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
                "no malformed reads detected".to_string(),
                None,
            ));
        }

        if matches!(metrics.pair_sync_checked, Some(true)) {
            let pair_integrity_ok = metrics.pair_sync_pass.unwrap_or(false)
                && metrics.pair_count_match.unwrap_or(false);
            if pair_integrity_ok {
                state.results.push(result(
                    "validate_pair_integrity",
                    bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
                    "paired validation preserved mate synchronization".to_string(),
                    None,
                ));
            } else {
                state.results.push(result(
                    "validate_pair_integrity",
                    bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
                    format!(
                        "pair validation failed with failure_class={}",
                        metrics.failure_class.clone().unwrap_or_else(|| "unknown".to_string())
                    ),
                    Some(
                        "inspect mate counts, header synchronization, and pair sync policy"
                            .to_string(),
                    ),
                ));
            }
        }

        if matches!(metrics.strict_pass, Some(false)) {
            state.results.push(result(
                "validate_strict_outcome",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
                format!(
                    "strict validation failed with failure_class={}",
                    metrics.failure_class.clone().unwrap_or_else(|| "unknown".to_string())
                ),
                Some("inspect governed validation report and input integrity".to_string()),
            ));
        } else if matches!(metrics.strict_pass, Some(true)) {
            state.results.push(result(
                "validate_strict_outcome",
                bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
                "strict validation completed without governed failures".to_string(),
                None,
            ));
        }
    } else {
        state.results.push(result(
            "metrics_parse",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
            "failed to parse validate metrics".to_string(),
            Some("verify metrics schema for fastq.validate_reads".to_string()),
        ));
    }
}
