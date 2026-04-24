use std::collections::BTreeMap;

use bijux_dna_analyze::{metric_semantics, MetricDirection};

use crate::policy::outcomes::{GateDecision, GateViolation};

use super::GatePolicy;

impl GatePolicy {
    #[allow(clippy::cast_precision_loss)]
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
            .map_or(self.required_metrics.as_slice(), |override_policy| {
                override_policy.required_metrics.as_slice()
            });
        let thresholds =
            overrides.map_or(&self.thresholds, |override_policy| &override_policy.thresholds);
        let allowed_regressions = overrides.map_or(&self.allowed_regressions, |override_policy| {
            &override_policy.allowed_regressions
        });
        let must_not_regress = overrides
            .map_or(self.must_not_regress.as_slice(), |override_policy| {
                override_policy.must_not_regress.as_slice()
            });
        let semantics_overrides = overrides.map_or(&self.semantics_overrides, |override_policy| {
            &override_policy.semantics_overrides
        });

        let mut violations = Vec::new();
        let mut missing_metrics = Vec::new();
        let mut rationale_trace = Vec::new();

        record_missing_required_metrics(
            required_metrics,
            metrics,
            &mut missing_metrics,
            &mut rationale_trace,
        );
        evaluate_thresholds(
            thresholds,
            semantics_overrides,
            metrics,
            &mut violations,
            &mut missing_metrics,
            &mut rationale_trace,
        );
        evaluate_regression_windows(
            allowed_regressions,
            semantics_overrides,
            metrics,
            &mut violations,
            &mut missing_metrics,
            &mut rationale_trace,
        );
        record_missing_regression_guards(
            must_not_regress,
            metrics,
            &mut missing_metrics,
            &mut rationale_trace,
        );

        GateDecision {
            schema_version: "bijux.bench.gate.v1".to_string(),
            dataset_id: dataset_id.to_string(),
            stage_id: stage_id.to_string(),
            tool_id: tool_id.to_string(),
            params_hash: params_hash.to_string(),
            passes: violations.is_empty() && missing_metrics.is_empty(),
            violations,
            missing_metrics: missing_metrics.clone(),
            completeness_score: completeness_score(required_metrics, &missing_metrics),
            rationale_trace,
        }
    }
}

fn record_missing_required_metrics(
    required_metrics: &[String],
    metrics: &BTreeMap<String, f64>,
    missing_metrics: &mut Vec<String>,
    rationale_trace: &mut Vec<String>,
) {
    for metric_id in required_metrics {
        if !metrics.contains_key(metric_id) {
            missing_metrics.push(metric_id.clone());
            rationale_trace.push(format!("missing_required:{metric_id}"));
        }
    }
}

fn evaluate_thresholds(
    thresholds: &BTreeMap<String, f64>,
    semantics_overrides: &BTreeMap<String, MetricDirection>,
    metrics: &BTreeMap<String, f64>,
    violations: &mut Vec<GateViolation>,
    missing_metrics: &mut Vec<String>,
    rationale_trace: &mut Vec<String>,
) {
    for (metric_id, threshold) in thresholds {
        let Some(semantics) = resolve_metric_direction(metric_id, semantics_overrides) else {
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
            MetricDirection::HigherBetter => *observed >= *threshold,
            MetricDirection::LowerBetter => *observed <= *threshold,
        };
        rationale_trace.push(format!("metric:{metric_id}:{observed} threshold:{threshold}"));
        if !passes {
            violations.push(GateViolation {
                metric_id: metric_id.clone(),
                observed: *observed,
                threshold: *threshold,
                direction: format!("{semantics:?}"),
            });
        }
    }
}

fn evaluate_regression_windows(
    allowed_regressions: &BTreeMap<String, f64>,
    semantics_overrides: &BTreeMap<String, MetricDirection>,
    metrics: &BTreeMap<String, f64>,
    violations: &mut Vec<GateViolation>,
    missing_metrics: &mut Vec<String>,
    rationale_trace: &mut Vec<String>,
) {
    for (metric_id, window) in allowed_regressions {
        let Some(semantics) = resolve_metric_direction(metric_id, semantics_overrides) else {
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
            MetricDirection::HigherBetter => *observed >= 1.0 - window,
            MetricDirection::LowerBetter => *observed <= 1.0 + window,
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
}

fn record_missing_regression_guards(
    must_not_regress: &[String],
    metrics: &BTreeMap<String, f64>,
    missing_metrics: &mut Vec<String>,
    rationale_trace: &mut Vec<String>,
) {
    for metric_id in must_not_regress {
        if !metrics.contains_key(metric_id) {
            missing_metrics.push(metric_id.clone());
            rationale_trace.push(format!("missing_must_not_regress:{metric_id}"));
        }
    }
}

fn resolve_metric_direction(
    metric_id: &str,
    semantics_overrides: &BTreeMap<String, MetricDirection>,
) -> Option<MetricDirection> {
    semantics_overrides
        .get(metric_id)
        .copied()
        .or_else(|| metric_semantics(metric_id).map(|semantics| semantics.direction))
}

fn completeness_score(required_metrics: &[String], missing_metrics: &[String]) -> f64 {
    if required_metrics.is_empty() {
        return 1.0;
    }
    let missing = f64::from(u32::try_from(missing_metrics.len()).unwrap_or(u32::MAX));
    let total = f64::from(u32::try_from(required_metrics.len()).unwrap_or(u32::MAX));
    (1.0 - (missing / total)).clamp(0.0, 1.0)
}
