//! Owner: bijux-analyze
//! Metric semantics resolution and normalization.
#![allow(dead_code)]

use anyhow::{anyhow, Result};

use bijux_runtime::MetricSemanticsV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricDirection {
    HigherBetter,
    LowerBetter,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricSemantics {
    pub metric_id: &'static str,
    pub direction: MetricDirection,
    pub units: &'static str,
    pub range: &'static str,
    pub missing_data_policy: &'static str,
    pub influencing_params: &'static [&'static str],
}

pub const COMPARE_METRIC_SEMANTICS: &[MetricSemantics] = &[
    MetricSemantics {
        metric_id: "runtime_s",
        direction: MetricDirection::LowerBetter,
        units: "seconds",
        range: ">= 0",
        missing_data_policy: "treat_as_infinite",
        influencing_params: &[],
    },
    MetricSemantics {
        metric_id: "memory_mb",
        direction: MetricDirection::LowerBetter,
        units: "MB",
        range: ">= 0",
        missing_data_policy: "treat_as_infinite",
        influencing_params: &[],
    },
    MetricSemantics {
        metric_id: "read_retention",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["adapter_bank", "trim_settings", "filter_settings"],
    },
    MetricSemantics {
        metric_id: "base_retention",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["adapter_bank", "trim_settings", "filter_settings"],
    },
    MetricSemantics {
        metric_id: "merge_rate",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["merge_policy"],
    },
    MetricSemantics {
        metric_id: "error_reduction_proxy",
        direction: MetricDirection::HigherBetter,
        units: "mean_q_delta",
        range: "[0, 45]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["corrector_settings"],
    },
];

#[must_use]
pub fn metric_semantics(metric_id: &str) -> Option<&'static MetricSemantics> {
    COMPARE_METRIC_SEMANTICS
        .iter()
        .find(|spec| spec.metric_id == metric_id)
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum MissingPolicy {
    TreatAsZero,
    TreatAsInfinite,
    Error,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NormalizedValue(pub f64);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticsError {
    pub metric_id: String,
    pub hint: String,
}

/// # Errors
/// Returns an error if the metric has no registered semantics entry.
pub fn resolve_semantics(metric_id: &str) -> Result<MetricSemanticsV1> {
    metric_semantics(metric_id)
        .map(|spec| MetricSemanticsV1 {
            metric_id: spec.metric_id.to_string(),
            direction: match spec.direction {
                MetricDirection::LowerBetter => "LowerBetter".to_string(),
                MetricDirection::HigherBetter => "HigherBetter".to_string(),
            },
            units: spec.units.to_string(),
            range: spec.range.to_string(),
            missing_data_policy: spec.missing_data_policy.to_string(),
            influencing_params: spec
                .influencing_params
                .iter()
                .map(|param| (*param).to_string())
                .collect(),
        })
        .ok_or_else(|| {
            anyhow!(
                "missing metric semantics for {metric_id}; register it in analyze semantics registry"
            )
        })
}

#[must_use]
pub fn normalize(value: f64, semantics: &MetricSemanticsV1) -> NormalizedValue {
    match semantics.direction.as_str() {
        "LowerBetter" => NormalizedValue(-value),
        _ => NormalizedValue(value),
    }
}

/// # Errors
/// Returns an error if the metric has no registered semantics entry.
pub fn missing_policy(metric_id: &str) -> Result<MissingPolicy> {
    let semantics = resolve_semantics(metric_id)?;
    let policy = match semantics.missing_data_policy.as_str() {
        "treat_as_0.0" => MissingPolicy::TreatAsZero,
        "treat_as_infinite" => MissingPolicy::TreatAsInfinite,
        _ => MissingPolicy::Error,
    };
    Ok(policy)
}
