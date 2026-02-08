//! Domain interfaces for pipeline definitions.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineSpec {
    pub stages: Vec<String>,
}

pub trait PipelineDomain {
    fn domain_id() -> &'static str;
    fn canonical_pipeline() -> PipelineSpec;
}
