use bijux_core::domain::PipelineDomain;

#[allow(dead_code)]
pub struct FastqDomain;

impl PipelineDomain for FastqDomain {
    fn domain_id() -> &'static str {
        "fastq"
    }

    fn canonical_pipeline() -> bijux_core::domain::PipelineSpec {
        crate::contracts::pipeline_contract::preprocess_pipeline()
    }
}
