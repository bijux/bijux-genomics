//! Owner: bijux-analyze
//! Decision core for ranking, comparison, and explainability.
//! Owns compare/score/explain logic and decision traces.
//! Must not depend on load/report or perform IO.
//! Invariants: missing semantics produce errors with remediation hints.

pub mod compare;
pub mod score;
mod score_helpers;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionMetricTrace {
    pub metric_id: String,
    pub value: Option<f64>,
    pub weight: f64,
    pub contribution: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionTrace {
    pub per_metric: Vec<DecisionMetricTrace>,
    pub penalties: Vec<String>,
    pub missing: Vec<String>,
    pub tie_breaks: Vec<String>,
}

impl DecisionTrace {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            per_metric: Vec::new(),
            penalties: Vec::new(),
            missing: Vec::new(),
            tie_breaks: Vec::new(),
        }
    }
}
