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
        let mut stages = Vec::new();
        for node in &nodes {
            if !stages.iter().any(|stage| stage == &node.stage_id) {
                stages.push(node.stage_id.clone());
            }
        }
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
            vec!["fastq.validate_reads".to_string(), "fastq.trim_reads".to_string()]
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
                },
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.cutadapt".to_string(),
                    to: "fastq.report_qc.compare".to_string(),
                },
            ],
        );
        assert_eq!(
            spec.stages,
            vec!["fastq.trim_reads".to_string(), "fastq.report_qc".to_string()]
        );
        assert!(spec.declares_graph_topology());
        assert_eq!(spec.nodes.len(), 3);
        assert_eq!(spec.edges.len(), 2);
    }
}
