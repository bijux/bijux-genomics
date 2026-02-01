#![allow(dead_code)]

use crate::domain::FastqStageKind;
use crate::params::{parse_effective_params, EffectiveParams};
use bijux_core::{
    FastqFilterMetricsV1, FastqMergeMetricsV1, FastqTrimMetricsV1, FastqValidateMetricsV1,
    InvariantResultV1, InvariantStatusV1, StageVerdictV1,
};

#[derive(Debug, Clone)]
pub struct InvariantThresholds {
    pub retention_warn: f64,
    pub retention_fail: f64,
    pub mean_q_delta_warn: f64,
    pub mean_q_delta_fail: f64,
    pub merge_rate_warn_low: f64,
    pub merge_rate_warn_high: f64,
    pub merge_rate_fail_low: f64,
    pub merge_rate_fail_high: f64,
    pub n_rate_warn: f64,
    pub n_rate_fail: f64,
}

impl Default for InvariantThresholds {
    fn default() -> Self {
        Self {
            retention_warn: 0.7,
            retention_fail: 0.4,
            mean_q_delta_warn: -1.0,
            mean_q_delta_fail: -3.0,
            merge_rate_warn_low: 0.1,
            merge_rate_warn_high: 0.9,
            merge_rate_fail_low: 0.05,
            merge_rate_fail_high: 0.98,
            n_rate_warn: 0.02,
            n_rate_fail: 0.05,
        }
    }
}

#[must_use]
pub fn thresholds_from_env() -> InvariantThresholds {
    fn parse_f64(key: &str, default: f64) -> f64 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(default)
    }
    let defaults = InvariantThresholds::default();
    InvariantThresholds {
        retention_warn: parse_f64("BIJUX_RETENTION_WARN", defaults.retention_warn),
        retention_fail: parse_f64("BIJUX_RETENTION_FAIL", defaults.retention_fail),
        mean_q_delta_warn: parse_f64("BIJUX_MEAN_Q_DELTA_WARN", defaults.mean_q_delta_warn),
        mean_q_delta_fail: parse_f64("BIJUX_MEAN_Q_DELTA_FAIL", defaults.mean_q_delta_fail),
        merge_rate_warn_low: parse_f64("BIJUX_MERGE_RATE_WARN_LOW", defaults.merge_rate_warn_low),
        merge_rate_warn_high: parse_f64(
            "BIJUX_MERGE_RATE_WARN_HIGH",
            defaults.merge_rate_warn_high,
        ),
        merge_rate_fail_low: parse_f64("BIJUX_MERGE_RATE_FAIL_LOW", defaults.merge_rate_fail_low),
        merge_rate_fail_high: parse_f64(
            "BIJUX_MERGE_RATE_FAIL_HIGH",
            defaults.merge_rate_fail_high,
        ),
        n_rate_warn: parse_f64("BIJUX_N_RATE_WARN", defaults.n_rate_warn),
        n_rate_fail: parse_f64("BIJUX_N_RATE_FAIL", defaults.n_rate_fail),
    }
}

#[derive(Debug, Clone)]
pub struct InvariantEvaluation {
    pub results: Vec<InvariantResultV1>,
    pub verdict: StageVerdictV1,
}

fn result(
    id: &str,
    status: InvariantStatusV1,
    message: String,
    remediation: Option<String>,
) -> InvariantResultV1 {
    InvariantResultV1 {
        id: id.to_string(),
        status,
        message,
        remediation,
    }
}

fn worst_status(current: InvariantStatusV1, next: &InvariantStatusV1) -> InvariantStatusV1 {
    std::cmp::max(current, next.clone())
}

fn retention_thresholds_for(
    params: Option<&EffectiveParams>,
    thresholds: &InvariantThresholds,
) -> (f64, f64) {
    let mut warn = thresholds.retention_warn;
    let mut fail = thresholds.retention_fail;
    let min_len = match params {
        Some(EffectiveParams::Trim(p)) => Some(p.min_len),
        Some(EffectiveParams::Merge(p)) => p.min_len,
        _ => None,
    };
    if let Some(min_len) = min_len {
        if min_len >= 100 {
            warn = (warn - 0.2).max(0.05);
            fail = (fail - 0.2).max(0.02);
        } else if min_len >= 50 {
            warn = (warn - 0.1).max(0.05);
            fail = (fail - 0.1).max(0.02);
        }
    }
    (warn, fail)
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn evaluate_invariants(
    stage_id: &str,
    metrics_json: &serde_json::Value,
    effective_params: &serde_json::Value,
    thresholds: &InvariantThresholds,
) -> InvariantEvaluation {
    let mut results = Vec::new();
    let parsed_params = parse_effective_params(stage_id, effective_params);
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

    let mut key_metrics = serde_json::Map::new();

    match stage_id {
        "fastq.trim" => {
            if let Ok(metrics) = serde_json::from_value::<FastqTrimMetricsV1>(metrics_json.clone())
            {
                key_metrics.insert(
                    "read_retention".to_string(),
                    serde_json::Value::from(metrics.delta_metrics.read_retention),
                );
                key_metrics.insert(
                    "base_retention".to_string(),
                    serde_json::Value::from(metrics.delta_metrics.base_retention),
                );
                key_metrics.insert(
                    "mean_q_delta".to_string(),
                    serde_json::Value::from(metrics.delta_metrics.mean_q_delta),
                );
                let (warn, fail) = retention_thresholds_for(parsed_params.as_ref(), thresholds);
                let retention = metrics.delta_metrics.read_retention;
                let remediation =
                    Some("review min_len/quality thresholds or input quality".to_string());
                if retention < fail {
                    results.push(result(
                        "retention_sanity",
                        InvariantStatusV1::Fail,
                        format!("read_retention {retention:.2} below fail threshold {fail:.2}"),
                        remediation,
                    ));
                } else if retention < warn {
                    results.push(result(
                        "retention_sanity",
                        InvariantStatusV1::Warn,
                        format!("read_retention {retention:.2} below warn threshold {warn:.2}"),
                        remediation,
                    ));
                } else {
                    results.push(result(
                        "retention_sanity",
                        InvariantStatusV1::Pass,
                        "read_retention within expected range".to_string(),
                        None,
                    ));
                }
                let q_delta = metrics.delta_metrics.mean_q_delta;
                if q_delta < thresholds.mean_q_delta_fail {
                    results.push(result(
                        "quality_direction",
                        InvariantStatusV1::Fail,
                        format!(
                            "mean_q_delta {:.2} below fail threshold {:.2}",
                            q_delta, thresholds.mean_q_delta_fail
                        ),
                        Some(
                            "review trimming settings; quality should not drop sharply".to_string(),
                        ),
                    ));
                } else if q_delta < thresholds.mean_q_delta_warn {
                    results.push(result(
                        "quality_direction",
                        InvariantStatusV1::Warn,
                        format!(
                            "mean_q_delta {:.2} below warn threshold {:.2}",
                            q_delta, thresholds.mean_q_delta_warn
                        ),
                        Some("review trimming settings for quality regression".to_string()),
                    ));
                } else {
                    results.push(result(
                        "quality_direction",
                        InvariantStatusV1::Pass,
                        "mean_q_delta within expected bounds".to_string(),
                        None,
                    ));
                }
            } else {
                results.push(result(
                    "metrics_parse",
                    InvariantStatusV1::Fail,
                    "failed to parse trim metrics".to_string(),
                    Some("verify metrics schema for fastq.trim".to_string()),
                ));
            }
        }
        "fastq.filter" => {
            if let Ok(metrics) =
                serde_json::from_value::<FastqFilterMetricsV1>(metrics_json.clone())
            {
                key_metrics.insert(
                    "read_retention".to_string(),
                    serde_json::Value::from(metrics.delta_metrics.read_retention),
                );
                key_metrics.insert(
                    "base_retention".to_string(),
                    serde_json::Value::from(metrics.delta_metrics.base_retention),
                );
                key_metrics.insert(
                    "mean_q_delta".to_string(),
                    serde_json::Value::from(metrics.delta_metrics.mean_q_delta),
                );
                let (warn, fail) = retention_thresholds_for(parsed_params.as_ref(), thresholds);
                let retention = metrics.delta_metrics.read_retention;
                let remediation = Some("review filtering thresholds or input quality".to_string());
                if retention < fail {
                    results.push(result(
                        "retention_sanity",
                        InvariantStatusV1::Fail,
                        format!("read_retention {retention:.2} below fail threshold {fail:.2}"),
                        remediation,
                    ));
                } else if retention < warn {
                    results.push(result(
                        "retention_sanity",
                        InvariantStatusV1::Warn,
                        format!("read_retention {retention:.2} below warn threshold {warn:.2}"),
                        remediation,
                    ));
                } else {
                    results.push(result(
                        "retention_sanity",
                        InvariantStatusV1::Pass,
                        "read_retention within expected range".to_string(),
                        None,
                    ));
                }
                let q_delta = metrics.delta_metrics.mean_q_delta;
                if q_delta < thresholds.mean_q_delta_fail {
                    results.push(result(
                        "quality_direction",
                        InvariantStatusV1::Fail,
                        format!(
                            "mean_q_delta {:.2} below fail threshold {:.2}",
                            q_delta, thresholds.mean_q_delta_fail
                        ),
                        Some("review filtering settings for quality regression".to_string()),
                    ));
                } else if q_delta < thresholds.mean_q_delta_warn {
                    results.push(result(
                        "quality_direction",
                        InvariantStatusV1::Warn,
                        format!(
                            "mean_q_delta {:.2} below warn threshold {:.2}",
                            q_delta, thresholds.mean_q_delta_warn
                        ),
                        Some("review filtering settings for quality regression".to_string()),
                    ));
                } else {
                    results.push(result(
                        "quality_direction",
                        InvariantStatusV1::Pass,
                        "mean_q_delta within expected bounds".to_string(),
                        None,
                    ));
                }
                if metrics.reads_in > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let n_rate = metrics.reads_removed_by_n as f64 / metrics.reads_in as f64;
                    key_metrics.insert("n_rate".to_string(), serde_json::Value::from(n_rate));
                    if n_rate > thresholds.n_rate_fail {
                        results.push(result(
                            "n_rate_sanity",
                            InvariantStatusV1::Fail,
                            format!(
                                "n_rate {:.3} exceeds fail threshold {:.3}",
                                n_rate, thresholds.n_rate_fail
                            ),
                            Some("review N filtering thresholds and input quality".to_string()),
                        ));
                    } else if n_rate > thresholds.n_rate_warn {
                        results.push(result(
                            "n_rate_sanity",
                            InvariantStatusV1::Warn,
                            format!(
                                "n_rate {:.3} exceeds warn threshold {:.3}",
                                n_rate, thresholds.n_rate_warn
                            ),
                            Some("review N filtering thresholds and input quality".to_string()),
                        ));
                    } else {
                        results.push(result(
                            "n_rate_sanity",
                            InvariantStatusV1::Pass,
                            "n_rate within expected bounds".to_string(),
                            None,
                        ));
                    }
                }
            } else {
                results.push(result(
                    "metrics_parse",
                    InvariantStatusV1::Fail,
                    "failed to parse filter metrics".to_string(),
                    Some("verify metrics schema for fastq.filter".to_string()),
                ));
            }
        }
        "fastq.validate_pre" => {
            if let Ok(metrics) =
                serde_json::from_value::<FastqValidateMetricsV1>(metrics_json.clone())
            {
                key_metrics.insert(
                    "reads_invalid".to_string(),
                    serde_json::Value::from(metrics.reads_invalid),
                );
                if metrics.reads_invalid > 0 {
                    results.push(result(
                        "validate_malformed_reads",
                        InvariantStatusV1::Fail,
                        format!("reads_invalid {} > 0", metrics.reads_invalid),
                        Some("inspect input FASTQ integrity and re-run validation".to_string()),
                    ));
                } else {
                    results.push(result(
                        "validate_malformed_reads",
                        InvariantStatusV1::Pass,
                        "no malformed reads detected".to_string(),
                        None,
                    ));
                }
            } else {
                results.push(result(
                    "metrics_parse",
                    InvariantStatusV1::Fail,
                    "failed to parse validate metrics".to_string(),
                    Some("verify metrics schema for fastq.validate_pre".to_string()),
                ));
            }
        }
        "fastq.merge" => {
            if let Ok(metrics) = serde_json::from_value::<FastqMergeMetricsV1>(metrics_json.clone())
            {
                key_metrics.insert(
                    "merge_rate".to_string(),
                    serde_json::Value::from(metrics.merge_rate),
                );
                let rate = metrics.merge_rate;
                if rate < thresholds.merge_rate_fail_low || rate > thresholds.merge_rate_fail_high {
                    results.push(result(
                        "merge_rate_range",
                        InvariantStatusV1::Fail,
                        format!(
                            "merge_rate {:.3} outside fail range [{:.2}, {:.2}]",
                            rate, thresholds.merge_rate_fail_low, thresholds.merge_rate_fail_high
                        ),
                        Some("inspect merge parameters and read overlap suitability".to_string()),
                    ));
                } else if rate < thresholds.merge_rate_warn_low
                    || rate > thresholds.merge_rate_warn_high
                {
                    results.push(result(
                        "merge_rate_range",
                        InvariantStatusV1::Warn,
                        format!(
                            "merge_rate {:.3} outside warn range [{:.2}, {:.2}]",
                            rate, thresholds.merge_rate_warn_low, thresholds.merge_rate_warn_high
                        ),
                        Some("inspect merge parameters and read overlap suitability".to_string()),
                    ));
                } else {
                    results.push(result(
                        "merge_rate_range",
                        InvariantStatusV1::Pass,
                        "merge_rate within expected range".to_string(),
                        None,
                    ));
                }
            } else {
                results.push(result(
                    "metrics_parse",
                    InvariantStatusV1::Fail,
                    "failed to parse merge metrics".to_string(),
                    Some("verify metrics schema for fastq.merge".to_string()),
                ));
            }
        }
        _ => {}
    }

    let mut verdict = InvariantStatusV1::Pass;
    let mut reasons = Vec::new();
    for entry in &results {
        verdict = worst_status(verdict, &entry.status);
        if entry.status != InvariantStatusV1::Pass {
            reasons.push(entry.id.clone());
        }
    }

    let stage_verdict = StageVerdictV1 {
        stage_id: stage_id.to_string(),
        verdict,
        reasons,
        key_metrics: serde_json::Value::Object(key_metrics),
    };

    InvariantEvaluation {
        results,
        verdict: stage_verdict,
    }
}

pub const CORE_STAGES: [&str; 6] = [
    "fastq.validate_pre",
    "fastq.trim",
    "fastq.merge",
    "fastq.correct",
    "fastq.filter",
    "fastq.stats_neutral",
];

pub const OPTIONAL_STAGES: [&str; 3] = ["fastq.qc_post", "fastq.umi", "fastq.screen"];

pub const META_STAGES: [&str; 1] = ["fastq.preprocess"];

pub const MUTATING_STAGES: [&str; 5] = [
    "fastq.trim",
    "fastq.merge",
    "fastq.correct",
    "fastq.filter",
    "fastq.umi",
];

pub const LOSSLESS_STAGES: [&str; 2] = ["fastq.validate_pre", "fastq.stats_neutral"];

pub const OBSERVATIONAL_STAGES: [&str; 4] = [
    "fastq.validate_pre",
    "fastq.stats_neutral",
    "fastq.qc_post",
    "fastq.screen",
];

#[must_use]
pub fn stage_kind(stage_id: &str) -> Option<FastqStageKind> {
    if CORE_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Core);
    }
    if OPTIONAL_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Optional);
    }
    if META_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Meta);
    }
    None
}
