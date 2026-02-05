use bijux_core::contract::PipelineDomain;

#[allow(dead_code)]
pub struct FastqDomain;

impl PipelineDomain for FastqDomain {
    fn domain_id() -> &'static str {
        "fastq"
    }

    fn canonical_pipeline() -> bijux_core::contract::PipelineSpec {
        crate::pipeline_contract::preprocess_pipeline()
    }
}
