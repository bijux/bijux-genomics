//! Domain interfaces for pipeline definitions.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PipelineNodeSpec {
    pub stage_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PipelineEdgeSpec {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_input_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineSpec {
    pub stages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<PipelineNodeSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<PipelineEdgeSpec>,
}

impl PipelineSpec {
    #[must_use]
    pub fn linear(stages: Vec<String>) -> Self {
        Self {
            stages,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    #[must_use]
    pub fn graph(nodes: Vec<PipelineNodeSpec>, edges: Vec<PipelineEdgeSpec>) -> Self {
        let stages = stage_catalog_from_nodes(&nodes);
        Self {
            stages,
            nodes,
            edges,
        }
    }

    #[must_use]
    pub fn declares_graph_topology(&self) -> bool {
        !self.nodes.is_empty() || !self.edges.is_empty()
    }

    #[must_use]
    pub fn ordered_nodes(&self) -> Vec<PipelineNodeSpec> {
        if !self.nodes.is_empty() {
            return self.nodes.clone();
        }
        self.stages
            .iter()
            .map(|stage_id| PipelineNodeSpec {
                stage_id: stage_id.clone(),
                stage_instance_id: None,
            })
            .collect()
    }

    #[must_use]
    pub fn stage_catalog(&self) -> Vec<String> {
        if self.declares_graph_topology() {
            return stage_catalog_from_nodes(&self.ordered_nodes());
        }
        self.stages.clone()
    }

    pub fn retain_nodes<F>(&mut self, mut keep: F)
    where
        F: FnMut(&PipelineNodeSpec) -> bool,
    {
        if !self.declares_graph_topology() {
            self.stages.retain(|stage_id| {
                keep(&PipelineNodeSpec {
                    stage_id: stage_id.clone(),
                    stage_instance_id: None,
                })
            });
            return;
        }

        self.nodes.retain(|node| keep(node));
        let retained_node_ids = self
            .nodes
            .iter()
            .map(|node| {
                PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref())
            })
            .collect::<std::collections::BTreeSet<_>>();
        self.edges.retain(|edge| {
            retained_node_ids.contains(&edge.from) && retained_node_ids.contains(&edge.to)
        });
        self.stages = stage_catalog_from_nodes(&self.nodes);
    }

    #[must_use]
    pub fn stage_node_id(stage_id: &str, stage_instance_id: Option<&str>) -> String {
        stage_instance_id.unwrap_or(stage_id).to_string()
    }
}

fn stage_catalog_from_nodes(nodes: &[PipelineNodeSpec]) -> Vec<String> {
    let mut stages = Vec::new();
    for node in nodes {
        if !stages.iter().any(|stage| stage == &node.stage_id) {
            stages.push(node.stage_id.clone());
        }
    }
    stages
}

pub trait PipelineDomain {
    fn domain_id() -> &'static str;
    fn canonical_pipeline() -> PipelineSpec;
}

#[cfg(test)]
mod tests {
    use super::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};

    #[test]
    fn linear_pipeline_spec_stays_compact() {
        let spec = PipelineSpec::linear(vec![
            "fastq.validate_reads".to_string(),
            "fastq.trim_reads".to_string(),
        ]);
        assert_eq!(
            spec.stages,
            vec![
                "fastq.validate_reads".to_string(),
                "fastq.trim_reads".to_string()
            ]
        );
        assert!(!spec.declares_graph_topology());
    }

    #[test]
    fn graph_pipeline_spec_preserves_unique_stage_catalog() {
        let spec = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.cutadapt".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.compare".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.fastp".to_string(),
                    to: "fastq.report_qc.compare".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.cutadapt".to_string(),
                    to: "fastq.report_qc.compare".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        );
        assert_eq!(
            spec.stage_catalog(),
            vec![
                "fastq.trim_reads".to_string(),
                "fastq.report_qc".to_string()
            ]
        );
        assert!(spec.declares_graph_topology());
        assert_eq!(spec.nodes.len(), 3);
        assert_eq!(spec.edges.len(), 2);
    }

    #[test]
    fn linear_pipeline_spec_can_materialize_ordered_nodes() {
        let spec = PipelineSpec::linear(vec![
            "fastq.validate_reads".to_string(),
            "fastq.trim_reads".to_string(),
        ]);
        let nodes = spec.ordered_nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].stage_id, "fastq.validate_reads");
        assert_eq!(nodes[0].stage_instance_id, None);
        assert_eq!(nodes[1].stage_id, "fastq.trim_reads");
        assert_eq!(nodes[1].stage_instance_id, None);
    }

    #[test]
    fn stage_node_id_prefers_instance_when_present() {
        assert_eq!(
            PipelineSpec::stage_node_id("fastq.trim_reads", Some("fastq.trim_reads.fastp")),
            "fastq.trim_reads.fastp"
        );
        assert_eq!(
            PipelineSpec::stage_node_id("fastq.trim_reads", None),
            "fastq.trim_reads"
        );
    }

    #[test]
    fn retain_nodes_prunes_graph_edges_and_catalog() {
        let mut spec = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validation".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.validate_reads.validation".to_string(),
                    to: "fastq.trim_reads.fastp".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.fastp".to_string(),
                    to: "fastq.report_qc.aggregate".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        );

        spec.retain_nodes(|node| node.stage_id != "fastq.validate_reads");

        assert_eq!(
            spec.stage_catalog(),
            vec![
                "fastq.trim_reads".to_string(),
                "fastq.report_qc".to_string()
            ]
        );
        assert_eq!(spec.nodes.len(), 2);
        assert_eq!(spec.edges.len(), 1);
        assert_eq!(spec.edges[0].from, "fastq.trim_reads.fastp");
    }
}
