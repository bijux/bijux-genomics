//! Owner: bijux-dna-bench-model
//! Benchmark suite graph expansion helpers.

use crate::model::graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind};

use super::{BenchmarkStageSpec, BenchmarkSuiteSpec};

impl BenchmarkStageSpec {
    #[must_use]
    pub fn stage_node_id(&self) -> &str {
        self.stage_instance_id.as_deref().unwrap_or(self.stage.as_str())
    }

    #[must_use]
    pub fn tool_node_id(&self, tool: &str) -> String {
        format!("{}.tool.{tool}", self.stage_node_id())
    }

    #[must_use]
    pub fn tool_node_ids(&self) -> Vec<String> {
        self.tools.iter().map(|tool| self.tool_node_id(tool)).collect()
    }

    #[must_use]
    pub fn stage_node(&self) -> BenchmarkGraphNode {
        BenchmarkGraphNode {
            node_id: self.stage_node_id().to_string(),
            kind: BenchmarkGraphNodeKind::Stage,
            stage_id: self.stage.clone(),
            stage_instance_id: self.stage_instance_id.clone(),
            tool_id: None,
        }
    }

    #[must_use]
    pub fn tool_node(&self, tool: &str) -> BenchmarkGraphNode {
        BenchmarkGraphNode {
            node_id: self.tool_node_id(tool),
            kind: BenchmarkGraphNodeKind::StageTool,
            stage_id: self.stage.clone(),
            stage_instance_id: self.stage_instance_id.clone(),
            tool_id: Some(tool.to_string()),
        }
    }

    #[must_use]
    pub fn graph_nodes(&self) -> Vec<BenchmarkGraphNode> {
        std::iter::once(self.stage_node())
            .chain(self.tools.iter().map(|tool| self.tool_node(tool)))
            .collect()
    }
}

impl BenchmarkSuiteSpec {
    #[must_use]
    pub fn stage_node_ids(&self) -> Vec<&str> {
        self.stages.iter().map(BenchmarkStageSpec::stage_node_id).collect()
    }

    #[must_use]
    pub fn stage_tool_node_ids(&self) -> Vec<String> {
        self.stages.iter().flat_map(BenchmarkStageSpec::tool_node_ids).collect()
    }

    #[must_use]
    pub fn graph_nodes(&self) -> Vec<BenchmarkGraphNode> {
        self.stages.iter().flat_map(BenchmarkStageSpec::graph_nodes).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{BenchmarkGraphNodeKind, BenchmarkStageSpec, BenchmarkSuiteSpec};
    use crate::model::suite::{
        AnalysisRequirements, DatasetSpec, DiversityRequirements, ReplicatePolicy,
        StratificationRequirement,
    };
    use bijux_dna_core::id_catalog;

    fn stage_instance(stage_id: &str, suffix: &str) -> String {
        format!("{stage_id}.{suffix}")
    }

    fn stage_tool_instance(stage_id: &str, suffix: &str, tool_id: &str) -> String {
        format!("{}.tool.{tool_id}", stage_instance(stage_id, suffix))
    }

    #[test]
    fn suite_graph_nodes_materialize_stage_and_stage_tool_identity() {
        let suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![BenchmarkStageSpec {
                stage: id_catalog::FASTQ_TRIM.to_string(),
                stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "cleanup")),
                tools: vec!["fastp".to_string(), "cutadapt".to_string()],
                params: Vec::new(),
                param_bindings: Vec::new(),
                upstream_stage_instance_ids: Vec::new(),
            }],
            ReplicatePolicy { count: 3, warmup: 0, seeds: vec![1, 2, 3] },
            DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );

        let nodes = suite.graph_nodes();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().any(|node| {
            node.kind == BenchmarkGraphNodeKind::Stage
                && node.node_id == stage_instance(id_catalog::FASTQ_TRIM, "cleanup")
                && node.tool_id.is_none()
        }));
        assert!(nodes.iter().any(|node| {
            node.kind == BenchmarkGraphNodeKind::StageTool
                && node.node_id
                    == stage_tool_instance(
                        id_catalog::FASTQ_TRIM,
                        "cleanup",
                        id_catalog::TOOL_FASTP,
                    )
                && node.tool_id.as_deref() == Some("fastp")
        }));
        assert!(nodes.iter().any(|node| {
            node.kind == BenchmarkGraphNodeKind::StageTool
                && node.node_id
                    == stage_tool_instance(
                        id_catalog::FASTQ_TRIM,
                        "cleanup",
                        id_catalog::TOOL_CUTADAPT,
                    )
                && node.tool_id.as_deref() == Some("cutadapt")
        }));
    }

    #[test]
    fn suite_graph_nodes_allow_planner_owned_stage_only_nodes() {
        let suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![BenchmarkStageSpec {
                stage: "benchmark.select_stage_tool".to_string(),
                stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
                tools: Vec::new(),
                params: Vec::new(),
                param_bindings: Vec::new(),
                upstream_stage_instance_ids: vec![
                    stage_instance(id_catalog::FASTQ_TRIM, "fastp"),
                    stage_instance(id_catalog::FASTQ_TRIM, "cutadapt"),
                ],
            }],
            ReplicatePolicy { count: 3, warmup: 0, seeds: vec![1, 2, 3] },
            DiversityRequirements { min_dataset_count: 1, min_classes: 1, min_read_layouts: 1 },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );

        let nodes = suite.graph_nodes();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].kind, BenchmarkGraphNodeKind::Stage);
        assert_eq!(nodes[0].node_id, "benchmark.select_stage_tool.trim_reads".to_string());
        assert!(nodes[0].tool_id.is_none());
    }
}
