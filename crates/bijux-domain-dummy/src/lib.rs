use bijux_core::domain::PipelineDomain;

pub struct DummyDomain;

impl PipelineDomain for DummyDomain {
    fn domain_id() -> &'static str {
        "dummy"
    }

    fn canonical_pipeline() -> bijux_core::domain::PipelineSpec {
        bijux_core::domain::PipelineSpec { stages: Vec::new() }
    }
}
