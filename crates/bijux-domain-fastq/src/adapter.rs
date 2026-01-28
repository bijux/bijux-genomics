use bijux_core::domain::PipelineDomain;

#[allow(dead_code)]
pub struct FastqDomain;

impl PipelineDomain for FastqDomain {
    fn domain_id() -> &'static str {
        "fastq"
    }

    fn canonical_pipeline() -> bijux_core::domain::PipelineSpec {
        let canonical = crate::pipeline::canonical_pipeline();
        bijux_core::domain::PipelineSpec {
            stages: canonical.required,
        }
    }
}
