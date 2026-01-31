//! Owner: bijux-analyze
//! Metric semantics resolution and normalization.
#![allow(dead_code)]

use anyhow::{anyhow, Result};

use bijux_core::metrics_registry::{metric_semantics, MetricDirection};
use bijux_core::MetricSemanticsV1;

pub mod metrics;

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
        })
        .ok_or_else(|| {
            anyhow!(
                "missing metric semantics for {metric_id}; register it in bijux-core metrics_registry"
            )
        })
}

pub fn normalize(value: f64, semantics: &MetricSemanticsV1) -> NormalizedValue {
    match semantics.direction.as_str() {
        "LowerBetter" => NormalizedValue(-value),
        _ => NormalizedValue(value),
    }
}

pub fn missing_policy(metric_id: &str) -> Result<MissingPolicy> {
    let semantics = resolve_semantics(metric_id)?;
    let policy = match semantics.missing_data_policy.as_str() {
        "treat_as_0.0" => MissingPolicy::TreatAsZero,
        "treat_as_infinite" => MissingPolicy::TreatAsInfinite,
        _ => MissingPolicy::Error,
    };
    Ok(policy)
}
