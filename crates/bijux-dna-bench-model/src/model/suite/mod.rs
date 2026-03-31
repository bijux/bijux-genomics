//! Owner: bijux-dna-bench
//! Benchmark suite specification (versioned).
//! Owns suite-level inputs for bench orchestration.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: `schema_version` is stable and versioned.

mod stage_graph;
mod support;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::graph::BenchmarkStageEdge;
pub use support::{
    AnalysisRequirements, DatasetSpec, DiversityRequirements, ReplicatePolicy,
    StratificationRequirement,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkParamBinding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageSpec {
    pub stage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    pub tools: Vec<String>,
    #[serde(default)]
    pub params: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_bindings: Vec<BenchmarkParamBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_stage_instance_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSuiteSpec {
    pub schema_version: String,
    pub suite_id: String,
    pub datasets: Vec<DatasetSpec>,
    pub stages: Vec<BenchmarkStageSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<BenchmarkStageEdge>,
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
        stages: &[String],
        tools: &[String],
        params: &[String],
        replicate_policy: ReplicatePolicy,
        diversity: DiversityRequirements,
        stratifications: Vec<StratificationRequirement>,
        analysis_requirements: AnalysisRequirements,
    ) -> Self {
        let stage_matrix = stages
            .iter()
            .cloned()
            .map(|stage| BenchmarkStageSpec {
                stage,
                stage_instance_id: None,
                tools: tools.to_vec(),
                params: params.to_vec(),
                param_bindings: Vec::new(),
                upstream_stage_instance_ids: Vec::new(),
            })
            .collect();
        Self::v1_stage_matrix(
            suite_id,
            datasets,
            stage_matrix,
            replicate_policy,
            diversity,
            stratifications,
            analysis_requirements,
        )
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn v1_stage_matrix(
        suite_id: String,
        datasets: Vec<DatasetSpec>,
        stages: Vec<BenchmarkStageSpec>,
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
            edges: Vec::new(),
            replicate_policy,
            diversity,
            stratifications,
            analysis_requirements,
        }
    }

    #[must_use]
    pub fn stage_ids(&self) -> Vec<&str> {
        self.stages
            .iter()
            .map(|stage| stage.stage.as_str())
            .collect()
    }
    #[must_use]
    pub fn tool_ids(&self) -> Vec<&str> {
        let mut tool_ids = std::collections::BTreeSet::new();
        for stage in &self.stages {
            for tool in &stage.tools {
                tool_ids.insert(tool.as_str());
            }
        }
        tool_ids.into_iter().collect()
    }

    #[must_use]
    pub fn params(&self) -> Vec<&str> {
        let mut params = std::collections::BTreeSet::new();
        for stage in &self.stages {
            for value in &stage.params {
                params.insert(value.as_str());
            }
            for binding in &stage.param_bindings {
                for key in binding.values.keys() {
                    params.insert(key.as_str());
                }
            }
        }
        params.into_iter().collect()
    }
}
