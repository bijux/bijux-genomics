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

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PipelineSpec {
    pub nodes: Vec<PipelineNodeSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<PipelineEdgeSpec>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PipelineSpecSerde {
    #[serde(default)]
    nodes: Vec<PipelineNodeSpec>,
    #[serde(default)]
    edges: Vec<PipelineEdgeSpec>,
}

impl<'de> serde::Deserialize<'de> for PipelineSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = PipelineSpecSerde::deserialize(deserializer)?;
        if repr.nodes.is_empty() {
            return Err(serde::de::Error::custom("PipelineSpec requires nodes"));
        }
        Ok(Self { nodes: repr.nodes, edges: repr.edges })
    }
}

impl PipelineSpec {
    #[must_use]
    pub fn graph(nodes: Vec<PipelineNodeSpec>, edges: Vec<PipelineEdgeSpec>) -> Self {
        Self { nodes, edges }
    }

    #[must_use]
    pub fn declares_graph_topology(&self) -> bool {
        !self.nodes.is_empty() || !self.edges.is_empty()
    }

    #[must_use]
    pub fn ordered_nodes(&self) -> Vec<PipelineNodeSpec> {
        self.nodes.clone()
    }

    #[must_use]
    pub fn ordered_stage_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.stage_id.clone()).collect()
    }

    #[must_use]
    pub fn stage_catalog(&self) -> Vec<String> {
        stage_catalog_from_nodes(&self.nodes)
    }

    pub fn retain_nodes<F>(&mut self, mut keep: F)
    where
        F: FnMut(&PipelineNodeSpec) -> bool,
    {
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

    fn stage_instance(stage_id: &str, suffix: &str) -> String {
        format!("{stage_id}.{suffix}")
    }

    fn explicit_linear_graph(stage_ids: &[&str]) -> PipelineSpec {
        PipelineSpec::graph(
            stage_ids
                .iter()
                .map(|stage_id| PipelineNodeSpec {
                    stage_id: (*stage_id).to_string(),
                    stage_instance_id: None,
                })
                .collect(),
            stage_ids
                .windows(2)
                .map(|window| PipelineEdgeSpec {
                    from: window[0].to_string(),
                    to: window[1].to_string(),
                    from_output_id: None,
                    to_input_id: None,
                })
                .collect(),
        )
    }

    #[test]
    fn graph_pipeline_spec_materializes_nodes_and_edges() {
        let spec = explicit_linear_graph(&[
            crate::id_catalog::FASTQ_VALIDATE_READS,
            crate::id_catalog::FASTQ_TRIM,
        ]);
        assert_eq!(spec.nodes.len(), 2);
        assert_eq!(spec.edges.len(), 1);
        assert!(spec.declares_graph_topology());
    }

    #[test]
    fn graph_pipeline_spec_preserves_unique_stage_catalog() {
        let spec = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: crate::id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp")),
                },
                PipelineNodeSpec {
                    stage_id: crate::id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(
                        crate::id_catalog::FASTQ_TRIM,
                        "cutadapt",
                    )),
                },
                PipelineNodeSpec {
                    stage_id: crate::id_catalog::FASTQ_QC_POST.to_string(),
                    stage_instance_id: Some(stage_instance(
                        crate::id_catalog::FASTQ_QC_POST,
                        "compare",
                    )),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp"),
                    to: stage_instance(crate::id_catalog::FASTQ_QC_POST, "compare"),
                    from_output_id: None,
                    to_input_id: None,
                },
                PipelineEdgeSpec {
                    from: stage_instance(crate::id_catalog::FASTQ_TRIM, "cutadapt"),
                    to: stage_instance(crate::id_catalog::FASTQ_QC_POST, "compare"),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        );
        assert_eq!(
            spec.stage_catalog(),
            vec![
                crate::id_catalog::FASTQ_TRIM.to_string(),
                crate::id_catalog::FASTQ_QC_POST.to_string(),
            ]
        );
        assert!(spec.declares_graph_topology());
        assert_eq!(spec.nodes.len(), 3);
        assert_eq!(spec.edges.len(), 2);
    }

    #[test]
    fn graph_pipeline_spec_can_materialize_ordered_nodes() {
        let spec = explicit_linear_graph(&[
            crate::id_catalog::FASTQ_VALIDATE_READS,
            crate::id_catalog::FASTQ_TRIM,
        ]);
        let nodes = spec.ordered_nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].stage_id, crate::id_catalog::FASTQ_VALIDATE_READS);
        assert_eq!(nodes[0].stage_instance_id, None);
        assert_eq!(nodes[1].stage_id, crate::id_catalog::FASTQ_TRIM);
        assert_eq!(nodes[1].stage_instance_id, None);
    }

    #[test]
    fn pipeline_spec_rejects_legacy_stage_lists() {
        let Err(error) = serde_json::from_value::<PipelineSpec>(serde_json::json!({
            "stages": [
                crate::id_catalog::FASTQ_VALIDATE_READS,
                crate::id_catalog::FASTQ_TRIM
            ]
        })) else {
            panic!("legacy stage lists should no longer deserialize");
        };
        assert!(error.to_string().contains("unknown field `stages`"));
    }

    #[test]
    fn stage_node_id_prefers_instance_when_present() {
        let trim_fastp = stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp");
        assert_eq!(
            PipelineSpec::stage_node_id(crate::id_catalog::FASTQ_TRIM, Some(trim_fastp.as_str())),
            trim_fastp
        );
        assert_eq!(
            PipelineSpec::stage_node_id(crate::id_catalog::FASTQ_TRIM, None),
            crate::id_catalog::FASTQ_TRIM
        );
    }

    #[test]
    fn retain_nodes_prunes_graph_edges_and_catalog() {
        let mut spec = PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: crate::id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    stage_instance_id: Some(stage_instance(
                        crate::id_catalog::FASTQ_VALIDATE_READS,
                        "validation",
                    )),
                },
                PipelineNodeSpec {
                    stage_id: crate::id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp")),
                },
                PipelineNodeSpec {
                    stage_id: crate::id_catalog::FASTQ_QC_POST.to_string(),
                    stage_instance_id: Some(stage_instance(
                        crate::id_catalog::FASTQ_QC_POST,
                        "aggregate",
                    )),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: stage_instance(crate::id_catalog::FASTQ_VALIDATE_READS, "validation"),
                    to: stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp"),
                    from_output_id: None,
                    to_input_id: None,
                },
                PipelineEdgeSpec {
                    from: stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp"),
                    to: stage_instance(crate::id_catalog::FASTQ_QC_POST, "aggregate"),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        );

        spec.retain_nodes(|node| node.stage_id != crate::id_catalog::FASTQ_VALIDATE_READS);

        assert_eq!(
            spec.stage_catalog(),
            vec![
                crate::id_catalog::FASTQ_TRIM.to_string(),
                crate::id_catalog::FASTQ_QC_POST.to_string(),
            ]
        );
        assert_eq!(spec.nodes.len(), 2);
        assert_eq!(spec.edges.len(), 1);
        assert_eq!(spec.edges[0].from, stage_instance(crate::id_catalog::FASTQ_TRIM, "fastp"));
    }
}
