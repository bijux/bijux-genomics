use crate::params::EffectiveParams;
use bijux_core::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};

#[derive(Debug, Clone)]
pub struct InvariantThresholds {
    pub retention_warn: f64,
    pub retention_fail: f64,
    pub mean_q_warn: f64,
    pub mean_q_fail: f64,
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
            mean_q_warn: 20.0,
            mean_q_fail: 15.0,
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
        mean_q_warn: parse_f64("BIJUX_MEAN_Q_WARN", defaults.mean_q_warn),
        mean_q_fail: parse_f64("BIJUX_MEAN_Q_FAIL", defaults.mean_q_fail),
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

pub(crate) fn result(
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

pub(crate) fn worst_status(
    current: InvariantStatusV1,
    next: &InvariantStatusV1,
) -> InvariantStatusV1 {
    std::cmp::max(current, next.clone())
}

pub(crate) fn retention_thresholds_for(
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
