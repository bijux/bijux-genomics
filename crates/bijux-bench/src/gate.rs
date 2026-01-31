#[must_use]
pub fn gate_passes(failure_count: usize) -> bool {
    failure_count == 0
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateViolation {
    pub metric_id: String,
    pub observed: f64,
    pub threshold: f64,
    pub direction: bijux_core::MetricSemanticsDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateDecision {
    pub passes: bool,
    pub violations: Vec<GateViolation>,
}

/// Evaluate metrics against thresholds using metric semantics.
///
/// # Panics
/// Panics if a threshold references a metric without known semantics.
#[must_use]
pub fn gate_with_thresholds(
    metrics: &serde_json::Value,
    thresholds: &std::collections::BTreeMap<String, f64>,
) -> GateDecision {
    let mut violations = Vec::new();
    for (metric_id, threshold) in thresholds {
        let semantics =
            bijux_core::metric_semantics(metric_id).unwrap_or_else(|| panic!("{metric_id}"));
        let Some(observed) = metrics.get(metric_id).and_then(|value| value.as_f64()) else {
            continue;
        };
        let passes = match semantics.direction {
            bijux_core::MetricSemanticsDirection::HigherBetter => observed >= *threshold,
            bijux_core::MetricSemanticsDirection::LowerBetter => observed <= *threshold,
        };
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
    }
}
