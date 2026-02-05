//! Owner: bijux-bench
//! Benchmark decision model (versioned).
//! Owns choice outputs and rationale trace.
//! Must not perform IO or depend on compare/gate logic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRationale {
    pub metric_id: String,
    pub observed: f64,
    pub direction: String,
    pub note: String,
    pub weight: f64,
    pub contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDecision {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub objective: String,
    pub passes: bool,
    pub rationale: Vec<DecisionRationale>,
    pub missing_metrics: Vec<String>,
}

impl BenchmarkDecision {
    #[must_use]
    pub fn v1(
        stage_id: String,
        tool_id: String,
        objective: String,
        passes: bool,
        rationale: Vec<DecisionRationale>,
        missing_metrics: Vec<String>,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.decision.v1".to_string(),
            stage_id,
            tool_id,
            objective,
            passes,
            rationale,
            missing_metrics,
        }
    }
}
