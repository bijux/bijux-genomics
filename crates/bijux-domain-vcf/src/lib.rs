//! Dummy VCF domain stub.

use bijux_core::domain::PipelineDomain;

pub struct VcfDomain;

impl PipelineDomain for VcfDomain {
    fn domain_id() -> &'static str {
        "vcf"
    }

    fn canonical_pipeline() -> bijux_core::domain::PipelineSpec {
        bijux_core::domain::PipelineSpec { stages: Vec::new() }
    }
}
