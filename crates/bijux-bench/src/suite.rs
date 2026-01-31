//! Owner: bijux-bench
//! Benchmark suite specification (versioned input contract).
//! Owns suite-level inputs for bench orchestration.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: schema_version is stable and versioned.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSuiteSpec {
    pub schema_version: String,
    pub suite_id: String,
    pub datasets: Vec<String>,
    pub stages: Vec<String>,
    pub tools: Vec<String>,
    pub params: Vec<String>,
    pub replicates: u32,
}

impl BenchmarkSuiteSpec {
    #[must_use]
    pub fn v1(
        suite_id: String,
        datasets: Vec<String>,
        stages: Vec<String>,
        tools: Vec<String>,
        params: Vec<String>,
        replicates: u32,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.suite.v1".to_string(),
            suite_id,
            datasets,
            stages,
            tools,
            params,
            replicates,
        }
    }
}
