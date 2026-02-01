//! Owner: bijux-bench
//! Gate policy engine for benchmark decisions.
//! Owns typed gating decisions with rationale trace.
//! Must not panic on missing metrics.
//! Invariants: decisions are deterministic.

use std::collections::BTreeMap;

use bijux_core::{metric_semantics, MetricSemanticsDirection};

use crate::error::BenchError;
use crate::policy::gate_decision::{GateDecision, GateViolation};

#[derive(Debug, Clone)]
pub struct GatePolicy {
    pub objective: String,
    pub required_metrics: Vec<String>,
    pub thresholds: BTreeMap<String, f64>,
    pub allowed_regressions: BTreeMap<String, f64>,
    pub must_not_regress: Vec<String>,
    pub semantics_overrides: BTreeMap<String, MetricSemanticsDirection>,
    pub stage_overrides: BTreeMap<String, GatePolicyOverrides>,
}

#[derive(Debug, Clone)]
pub struct GatePolicyOverrides {
    pub required_metrics: Vec<String>,
    pub thresholds: BTreeMap<String, f64>,
    pub allowed_regressions: BTreeMap<String, f64>,
    pub must_not_regress: Vec<String>,
    pub semantics_overrides: BTreeMap<String, MetricSemanticsDirection>,
}

impl GatePolicy {
    /// # Errors
    /// Returns an error if the policy references unknown metrics.
    pub fn validate(&self) -> Result<(), BenchError> {
        let mut unknown = Vec::new();
        for metric_id in self
            .required_metrics
            .iter()
            .chain(self.thresholds.keys())
            .chain(self.allowed_regressions.keys())
            .chain(self.must_not_regress.iter())
        {
            if metric_semantics(metric_id).is_none()
                && !self.semantics_overrides.contains_key(metric_id)
            {
                unknown.push(metric_id.clone());
            }
        }
        for override_policy in self.stage_overrides.values() {
            for metric_id in override_policy
                .required_metrics
                .iter()
                .chain(override_policy.thresholds.keys())
                .chain(override_policy.allowed_regressions.keys())
                .chain(override_policy.must_not_regress.iter())
            {
                if metric_semantics(metric_id).is_none()
                    && !override_policy.semantics_overrides.contains_key(metric_id)
                {
                    unknown.push(metric_id.clone());
                }
            }
        }
        if !unknown.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "unknown metrics: {}",
                unknown.join(",")
            )));
        }
        Ok(())
    }

    #[must_use]
    pub fn decide(
        &self,
        dataset_id: &str,
        stage_id: &str,
        tool_id: &str,
        params_hash: &str,
        metrics: &BTreeMap<String, f64>,
    ) -> GateDecision {
        let overrides = Some(stage_id).and_then(|stage| self.stage_overrides.get(stage));
        let required_metrics = overrides
            .map(|override_policy| override_policy.required_metrics.as_slice())
            .unwrap_or(self.required_metrics.as_slice());
        let thresholds = overrides
            .map(|override_policy| &override_policy.thresholds)
            .unwrap_or(&self.thresholds);
        let allowed_regressions = overrides
            .map(|override_policy| &override_policy.allowed_regressions)
            .unwrap_or(&self.allowed_regressions);
        let must_not_regress = overrides
            .map(|override_policy| override_policy.must_not_regress.as_slice())
            .unwrap_or(self.must_not_regress.as_slice());
        let semantics_overrides = overrides
            .map(|override_policy| &override_policy.semantics_overrides)
            .unwrap_or(&self.semantics_overrides);

        let mut violations = Vec::new();
        let mut missing_metrics = Vec::new();
        let mut rationale_trace = Vec::new();

        for metric_id in required_metrics {
            if !metrics.contains_key(metric_id) {
                missing_metrics.push(metric_id.to_string());
                rationale_trace.push(format!("missing_required:{metric_id}"));
            }
        }

        for (metric_id, threshold) in thresholds {
            let semantics = semantics_overrides
                .get(metric_id)
                .copied()
                .or_else(|| metric_semantics(metric_id).map(|s| s.direction));
            let Some(semantics) = semantics else {
                missing_metrics.push(metric_id.clone());
                rationale_trace.push(format!("missing_semantics:{metric_id}"));
                continue;
            };
            let Some(observed) = metrics.get(metric_id) else {
                missing_metrics.push(metric_id.clone());
                rationale_trace.push(format!("missing_metric:{metric_id}"));
                continue;
            };
            let passes = match semantics {
                MetricSemanticsDirection::HigherBetter => *observed >= *threshold,
                MetricSemanticsDirection::LowerBetter => *observed <= *threshold,
            };
            rationale_trace.push(format!(
                "metric:{metric_id}:{observed} threshold:{threshold}"
            ));
            if !passes {
                violations.push(GateViolation {
                    metric_id: metric_id.clone(),
                    observed: *observed,
                    threshold: *threshold,
                    direction: format!("{semantics:?}"),
                });
            }
        }

        for (metric_id, window) in allowed_regressions {
            let semantics = semantics_overrides
                .get(metric_id)
                .copied()
                .or_else(|| metric_semantics(metric_id).map(|s| s.direction));
            let Some(semantics) = semantics else {
                missing_metrics.push(metric_id.clone());
                rationale_trace.push(format!("missing_semantics:{metric_id}"));
                continue;
            };
            let Some(observed) = metrics.get(metric_id) else {
                missing_metrics.push(metric_id.clone());
                rationale_trace.push(format!("missing_metric:{metric_id}"));
                continue;
            };
            let passes = match semantics {
                MetricSemanticsDirection::HigherBetter => *observed >= 1.0 - window,
                MetricSemanticsDirection::LowerBetter => *observed <= 1.0 + window,
            };
            rationale_trace.push(format!("window:{metric_id}:{observed} limit:{window}"));
            if !passes {
                violations.push(GateViolation {
                    metric_id: metric_id.clone(),
                    observed: *observed,
                    threshold: *window,
                    direction: format!("{semantics:?}"),
                });
            }
        }

        for metric_id in must_not_regress {
            if !metrics.contains_key(metric_id) {
                missing_metrics.push(metric_id.to_string());
                rationale_trace.push(format!("missing_must_not_regress:{metric_id}"));
            }
        }

        let completeness_score = if required_metrics.is_empty() {
            1.0
        } else {
            let missing = missing_metrics.len() as f64;
            let total = required_metrics.len() as f64;
            (1.0 - (missing / total)).clamp(0.0, 1.0)
        };

        GateDecision {
            schema_version: "bijux.bench.gate.v1".to_string(),
            dataset_id: dataset_id.to_string(),
            stage_id: stage_id.to_string(),
            tool_id: tool_id.to_string(),
            params_hash: params_hash.to_string(),
            passes: violations.is_empty() && missing_metrics.is_empty(),
            violations,
            missing_metrics,
            completeness_score,
            rationale_trace,
        }
    }
}
