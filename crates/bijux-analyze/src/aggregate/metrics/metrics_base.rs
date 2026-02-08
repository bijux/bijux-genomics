//! Owner: bijux-analyze
//! Shared metric schema helpers and exports.

use bijux_core::metrics::MetricSet;
use bijux_core::prelude::measure::ExecutionMetrics;
use serde::{Deserialize, Serialize};

use crate::aggregate::schema::{metric_spec, StageMetricKind, StageMetricRegistry};
use crate::aggregate::{BenchError, Result};

pub use super::bench::*;
pub use super::fastq::*;

pub trait StageMetricSchema {
    const STAGE: &'static str;
    const VERSION: i32;
    /// Validate the schema invariants.
    ///
    /// # Errors
    /// Returns an error if the schema invariants are violated.
    fn validate(&self) -> Result<()>;
}

fn metric_schema_name(stage: &str, version: i32) -> String {
    format!("{}_v{}", stage.replace('.', "_"), version)
}

fn validate_metric_schema<T>(metrics: &T) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let stage = T::STAGE;
    let spec = StageMetricRegistry::spec_for_stage(stage).ok_or_else(|| {
        BenchError::Validation(format!("missing metric schema for stage {stage}"))
    })?;
    let value = serde_json::to_value(metrics)?;
    let obj = value
        .as_object()
        .ok_or_else(|| BenchError::Validation("metrics must serialize to object".to_string()))?;
    let observed: std::collections::BTreeSet<String> = obj
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let expected: std::collections::BTreeSet<String> = spec
        .metrics
        .iter()
        .map(|metric_id| metric_spec(*metric_id).name.to_string())
        .collect();
    if observed != expected {
        return Err(BenchError::Validation(format!(
            "metric schema mismatch for {stage}: observed={observed:?} expected={expected:?}",
        )));
    }
    Ok(())
}

pub fn metric_set<T>(metrics: T) -> MetricSet<T>
where
    T: StageMetricSchema + Serialize,
{
    MetricSet::new(
        metric_schema_name(T::STAGE, T::VERSION),
        T::VERSION,
        metrics,
    )
}

/// Validate the metric set.
///
/// # Errors
/// Returns an error if the metric schema validation fails.
pub fn validate_metric_set<T>(set: &MetricSet<T>) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let expected_schema = metric_schema_name(T::STAGE, T::VERSION);
    if set.metrics_schema != expected_schema {
        return Err(BenchError::Validation(format!(
            "metric schema mismatch for {}: expected {} got {}",
            T::STAGE,
            expected_schema,
            set.metrics_schema
        )));
    }
    if set.version != T::VERSION {
        return Err(BenchError::Validation(format!(
            "metric version mismatch for {}: expected {} got {}",
            T::STAGE,
            T::VERSION,
            set.version
        )));
    }
    set.metrics.validate()?;
    validate_metric_schema(&set.metrics)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankingInput<T: StageMetricSchema> {
    pub stage: StageMetricKind,
    pub execution: ExecutionMetrics,
    pub metrics: T,
}

#[must_use]
pub fn normalize_rate(value: f64) -> Option<f64> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

#[must_use]
pub fn normalize_inverse_rate(value: f64) -> Option<f64> {
    normalize_rate(value).map(|v| 1.0 - v)
}
