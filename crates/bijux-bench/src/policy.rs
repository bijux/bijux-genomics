//! Owner: bijux-bench
//! Gate policy engine for benchmark decisions.
//! Owns typed gating decisions with rationale trace.
//! Must not panic on missing metrics.
//! Invariants: decisions are deterministic.

use std::collections::BTreeMap;

use bijux_core::{metric_semantics, MetricSemanticsDirection};

#[derive(Debug, Clone)]
pub struct GatePolicy {
    pub required_metrics: Vec<String>,
    pub thresholds: BTreeMap<String, f64>,
    pub regression_windows: BTreeMap<String, f64>,
    pub stage_overrides: BTreeMap<String, GatePolicyOverrides>,
}

#[derive(Debug, Clone)]
pub struct GatePolicyOverrides {
    pub required_metrics: Vec<String>,
    pub thresholds: BTreeMap<String, f64>,
    pub regression_windows: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateViolation {
    pub metric_id: String,
    pub observed: f64,
    pub threshold: f64,
    pub direction: MetricSemanticsDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateDecision {
    pub passes: bool,
    pub violations: Vec<GateViolation>,
    pub missing_metrics: Vec<String>,
    pub trace: Vec<String>,
}

impl GatePolicy {
    #[must_use]
    pub fn decide(&self, stage_id: Option<&str>, metrics: &serde_json::Value) -> GateDecision {
        let overrides = stage_id.and_then(|stage| self.stage_overrides.get(stage));
        let required_metrics = overrides
            .map(|override_policy| override_policy.required_metrics.as_slice())
            .unwrap_or(self.required_metrics.as_slice());
        let thresholds = overrides
            .map(|override_policy| &override_policy.thresholds)
            .unwrap_or(&self.thresholds);
        let regression_windows = overrides
            .map(|override_policy| &override_policy.regression_windows)
            .unwrap_or(&self.regression_windows);

        let mut violations = Vec::new();
        let mut missing_metrics = Vec::new();
        let mut trace = Vec::new();
        for metric_id in required_metrics {
            if metrics
                .get(metric_id)
                .and_then(|value| value.as_f64())
                .is_none()
            {
                missing_metrics.push(metric_id.to_string());
                trace.push(format!("missing_required:{metric_id}"));
            }
        }

        for (metric_id, threshold) in thresholds {
            let Some(semantics) = metric_semantics(metric_id) else {
                missing_metrics.push(metric_id.clone());
                trace.push(format!("missing_semantics:{metric_id}"));
                continue;
            };
            let Some(observed) = metrics.get(metric_id).and_then(|value| value.as_f64()) else {
                missing_metrics.push(metric_id.clone());
                trace.push(format!("missing_metric:{metric_id}"));
                continue;
            };
            let passes = match semantics.direction {
                MetricSemanticsDirection::HigherBetter => observed >= *threshold,
                MetricSemanticsDirection::LowerBetter => observed <= *threshold,
            };
            trace.push(format!(
                "metric:{metric_id}:{observed} threshold:{threshold}"
            ));
            if !passes {
                violations.push(GateViolation {
                    metric_id: metric_id.clone(),
                    observed,
                    threshold: *threshold,
                    direction: semantics.direction,
                });
            }
        }

        for (metric_id, window) in regression_windows {
            let Some(semantics) = metric_semantics(metric_id) else {
                missing_metrics.push(metric_id.clone());
                trace.push(format!("missing_semantics:{metric_id}"));
                continue;
            };
            let Some(observed) = metrics.get(metric_id).and_then(|value| value.as_f64()) else {
                missing_metrics.push(metric_id.clone());
                trace.push(format!("missing_metric:{metric_id}"));
                continue;
            };
            let passes = match semantics.direction {
                MetricSemanticsDirection::HigherBetter => observed >= 1.0 - window,
                MetricSemanticsDirection::LowerBetter => observed <= 1.0 + window,
            };
            trace.push(format!("window:{metric_id}:{observed} limit:{window}"));
            if !passes {
                violations.push(GateViolation {
                    metric_id: metric_id.clone(),
                    observed,
                    threshold: *window,
                    direction: semantics.direction,
                });
            }
        }
        GateDecision {
            passes: violations.is_empty(),
            violations,
            missing_metrics,
            trace,
        }
    }
}
