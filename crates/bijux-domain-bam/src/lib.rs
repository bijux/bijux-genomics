//! Dummy BAM domain stub.

use bijux_core::domain::PipelineDomain;

pub struct BamDomain;

impl PipelineDomain for BamDomain {
    fn domain_id() -> &'static str {
        "bam"
    }

    fn canonical_pipeline() -> bijux_core::domain::PipelineSpec {
        bijux_core::domain::PipelineSpec { stages: Vec::new() }
    }
}
