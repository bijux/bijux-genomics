//! Owner: bijux-benchmark
//! Benchmark suite specification (versioned).
//! Owns suite-level inputs for bench orchestration.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: schema_version is stable and versioned.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSpec {
    pub id: String,
    pub hash: String,
    pub size: u64,
    pub origin: String,
    pub class_label: String,
    pub read_layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicatePolicy {
    pub count: u32,
    pub warmup: u32,
    pub seeds: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiversityRequirements {
    pub min_dataset_count: usize,
    pub min_classes: usize,
    pub min_read_layouts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StratificationRequirement {
    pub key: String,
    pub required_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequirements {
    pub require_bootstrap: bool,
    pub require_outlier_detection: bool,
    pub min_replicates_for_bootstrap: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSuiteSpec {
    pub schema_version: String,
    pub suite_id: String,
    pub datasets: Vec<DatasetSpec>,
    pub stages: Vec<String>,
    pub tools: Vec<String>,
    pub params: Vec<String>,
    pub replicate_policy: ReplicatePolicy,
    pub diversity: DiversityRequirements,
    pub stratifications: Vec<StratificationRequirement>,
    pub analysis_requirements: AnalysisRequirements,
}

impl BenchmarkSuiteSpec {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        suite_id: String,
        datasets: Vec<DatasetSpec>,
        stages: Vec<String>,
        tools: Vec<String>,
        params: Vec<String>,
        replicate_policy: ReplicatePolicy,
        diversity: DiversityRequirements,
        stratifications: Vec<StratificationRequirement>,
        analysis_requirements: AnalysisRequirements,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.suite.v1".to_string(),
            suite_id,
            datasets,
            stages,
            tools,
            params,
            replicate_policy,
            diversity,
            stratifications,
            analysis_requirements,
        }
    }
}
