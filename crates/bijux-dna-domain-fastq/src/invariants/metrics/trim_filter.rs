use crate::metrics::{FastqFilterMetricsV1, FastqTrimMetricsV1};

use super::shared::EvaluationState;
use crate::invariants::evaluation::{result, retention_thresholds_for, InvariantThresholds};

pub(super) fn evaluate_trim(
    state: &mut EvaluationState,
    metrics_json: &serde_json::Value,
    thresholds: &InvariantThresholds,
) {
    if let Ok(metrics) = serde_json::from_value::<FastqTrimMetricsV1>(metrics_json.clone()) {
        state.key_metrics.insert(
            "read_retention".to_string(),
            serde_json::Value::from(metrics.delta_metrics.read_retention),
        );
        state.key_metrics.insert(
            "base_retention".to_string(),
            serde_json::Value::from(metrics.delta_metrics.base_retention),
        );
        state.key_metrics.insert(
            "mean_q_delta".to_string(),
            serde_json::Value::from(metrics.delta_metrics.mean_q_delta),
        );

        let (warn, fail) = retention_thresholds_for(state.parsed_params.as_ref(), thresholds);
        push_retention_result(
            &mut state.results,
            metrics.delta_metrics.read_retention,
            warn,
            fail,
            "review min_len/quality thresholds or input quality",
        );
        push_quality_result(
            &mut state.results,
            metrics.delta_metrics.mean_q_delta,
            thresholds,
            "review trimming settings; quality should not drop sharply",
            "review trimming settings for quality regression",
        );
    } else {
        state.results.push(result(
            "metrics_parse",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
            "failed to parse trim metrics".to_string(),
            Some("verify metrics schema for fastq.trim_reads".to_string()),
        ));
    }
}

pub(super) fn evaluate_filter(
    state: &mut EvaluationState,
    metrics_json: &serde_json::Value,
    thresholds: &InvariantThresholds,
) {
    if let Ok(metrics) = serde_json::from_value::<FastqFilterMetricsV1>(metrics_json.clone()) {
        state.key_metrics.insert(
            "read_retention".to_string(),
            serde_json::Value::from(metrics.delta_metrics.read_retention),
        );
        state.key_metrics.insert(
            "base_retention".to_string(),
            serde_json::Value::from(metrics.delta_metrics.base_retention),
        );
        state.key_metrics.insert(
            "mean_q_delta".to_string(),
            serde_json::Value::from(metrics.delta_metrics.mean_q_delta),
        );

        let (warn, fail) = retention_thresholds_for(state.parsed_params.as_ref(), thresholds);
        push_retention_result(
            &mut state.results,
            metrics.delta_metrics.read_retention,
            warn,
            fail,
            "review filtering thresholds or input quality",
        );
        push_quality_result(
            &mut state.results,
            metrics.delta_metrics.mean_q_delta,
            thresholds,
            "review filtering settings for quality regression",
            "review filtering settings for quality regression",
        );

        if metrics.reads_in > 0 {
            #[allow(clippy::cast_precision_loss)]
            let n_rate = metrics.reads_removed_by_n as f64 / metrics.reads_in as f64;
            state.key_metrics.insert("n_rate".to_string(), serde_json::Value::from(n_rate));
            if n_rate > thresholds.n_rate_fail {
                state.results.push(result(
                    "n_rate_sanity",
                    bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
                    format!(
                        "n_rate {:.3} exceeds fail threshold {:.3}",
                        n_rate, thresholds.n_rate_fail
                    ),
                    Some("review N filtering thresholds and input quality".to_string()),
                ));
            } else if n_rate > thresholds.n_rate_warn {
                state.results.push(result(
                    "n_rate_sanity",
                    bijux_dna_core::prelude::invariants::InvariantStatusV1::Warn,
                    format!(
                        "n_rate {:.3} exceeds warn threshold {:.3}",
                        n_rate, thresholds.n_rate_warn
                    ),
                    Some("review N filtering thresholds and input quality".to_string()),
                ));
            } else {
                state.results.push(result(
                    "n_rate_sanity",
                    bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
                    "n_rate within expected bounds".to_string(),
                    None,
                ));
            }
        }
    } else {
        state.results.push(result(
            "metrics_parse",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
            "failed to parse filter metrics".to_string(),
            Some("verify metrics schema for fastq.filter_reads".to_string()),
        ));
    }
}

fn push_retention_result(
    results: &mut Vec<bijux_dna_core::prelude::invariants::InvariantResultV1>,
    retention: f64,
    warn: f64,
    fail: f64,
    remediation: &str,
) {
    if retention < fail {
        results.push(result(
            "retention_sanity",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
            format!("read_retention {retention:.2} below fail threshold {fail:.2}"),
            Some(remediation.to_string()),
        ));
    } else if retention < warn {
        results.push(result(
            "retention_sanity",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Warn,
            format!("read_retention {retention:.2} below warn threshold {warn:.2}"),
            Some(remediation.to_string()),
        ));
    } else {
        results.push(result(
            "retention_sanity",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
            "read_retention within expected range".to_string(),
            None,
        ));
    }
}

fn push_quality_result(
    results: &mut Vec<bijux_dna_core::prelude::invariants::InvariantResultV1>,
    q_delta: f64,
    thresholds: &InvariantThresholds,
    fail_remediation: &str,
    warn_remediation: &str,
) {
    if q_delta < thresholds.mean_q_delta_fail {
        results.push(result(
            "quality_direction",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Fail,
            format!(
                "mean_q_delta {:.2} below fail threshold {:.2}",
                q_delta, thresholds.mean_q_delta_fail
            ),
            Some(fail_remediation.to_string()),
        ));
    } else if q_delta < thresholds.mean_q_delta_warn {
        results.push(result(
            "quality_direction",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Warn,
            format!(
                "mean_q_delta {:.2} below warn threshold {:.2}",
                q_delta, thresholds.mean_q_delta_warn
            ),
            Some(warn_remediation.to_string()),
        ));
    } else {
        results.push(result(
            "quality_direction",
            bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass,
            "mean_q_delta within expected bounds".to_string(),
            None,
        ));
    }
}
