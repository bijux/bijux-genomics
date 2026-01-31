//! Owner: bijux-bench
//! Gate decision outputs.

use bijux_core::MetricSemanticsDirection;

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
    pub completeness_score: f64,
    pub rationale_trace: Vec<String>,
}
