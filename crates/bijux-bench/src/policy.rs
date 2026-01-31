//! Owner: bijux-bench
//! Gate policy engine for benchmark decisions.
//! Owns typed gating decisions with rationale trace.
//! Must not panic on missing metrics.
//! Invariants: decisions are deterministic.

use std::collections::BTreeMap;

use bijux_core::{metric_semantics, MetricSemanticsDirection};

#[derive(Debug, Clone)]
pub struct GatePolicy {
    pub thresholds: BTreeMap<String, f64>,
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
    pub missing: Vec<String>,
    pub trace: Vec<String>,
}

impl GatePolicy {
    #[must_use]
    pub fn decide(&self, metrics: &serde_json::Value) -> GateDecision {
        let mut violations = Vec::new();
        let mut missing = Vec::new();
        let mut trace = Vec::new();
        for (metric_id, threshold) in &self.thresholds {
            let Some(semantics) = metric_semantics(metric_id) else {
                missing.push(metric_id.clone());
                trace.push(format!("missing_semantics:{metric_id}"));
                continue;
            };
            let Some(observed) = metrics.get(metric_id).and_then(|value| value.as_f64()) else {
                missing.push(metric_id.clone());
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
        GateDecision {
            passes: violations.is_empty(),
            violations,
            missing,
            trace,
        }
    }
}
