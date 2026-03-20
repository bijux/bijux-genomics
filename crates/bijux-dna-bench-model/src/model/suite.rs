//! Owner: bijux-dna-bench
//! Benchmark suite specification (versioned).
//! Owns suite-level inputs for bench orchestration.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: `schema_version` is stable and versioned.

use std::collections::BTreeMap;

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

impl BenchmarkStageSpec {
    #[must_use]
    pub fn stage_node_id(&self) -> &str {
        self.stage_instance_id
            .as_deref()
            .unwrap_or(self.stage.as_str())
    }

    #[must_use]
    pub fn tool_node_id(&self, tool: &str) -> String {
        format!("{}.tool.{tool}", self.stage_node_id())
    }

    #[must_use]
    pub fn tool_node_ids(&self) -> Vec<String> {
        self.tools
            .iter()
            .map(|tool| self.tool_node_id(tool))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageEdge {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_input_id: Option<String>,
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
        stages: Vec<String>,
        tools: Vec<String>,
        params: Vec<String>,
        replicate_policy: ReplicatePolicy,
        diversity: DiversityRequirements,
        stratifications: Vec<StratificationRequirement>,
        analysis_requirements: AnalysisRequirements,
    ) -> Self {
        let stage_matrix = stages
            .into_iter()
            .map(|stage| BenchmarkStageSpec {
                stage,
                stage_instance_id: None,
                tools: tools.clone(),
                params: params.clone(),
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
    pub fn stage_node_ids(&self) -> Vec<&str> {
        self.stages.iter().map(BenchmarkStageSpec::stage_node_id).collect()
    }

    #[must_use]
    pub fn stage_tool_node_ids(&self) -> Vec<String> {
        self.stages
            .iter()
            .flat_map(BenchmarkStageSpec::tool_node_ids)
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
