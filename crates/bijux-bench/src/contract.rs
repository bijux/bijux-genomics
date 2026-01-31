//! Owner: bijux-bench
//! Contracted schema for bench artifacts.
//! Owns stable public output types.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: schema_version is stable and versioned.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSummary {
    pub schema_version: String,
    pub suite_id: String,
    pub dataset_hash: String,
    pub observations: usize,
    pub warnings: Vec<String>,
    pub decisions: Vec<BenchmarkDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDecision {
    pub tool: String,
    pub passes: bool,
    pub missing_metrics: Vec<String>,
    pub rationale: Vec<String>,
}

impl BenchmarkSummary {
    #[must_use]
    pub fn v1(
        suite_id: String,
        dataset_hash: String,
        observations: usize,
        warnings: Vec<String>,
        decisions: Vec<BenchmarkDecision>,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.summary.v1".to_string(),
            suite_id,
            dataset_hash,
            observations,
            warnings,
            decisions,
        }
    }
}
