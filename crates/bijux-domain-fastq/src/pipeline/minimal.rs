use bijux_engine::api::PipelineSpec;

use super::canonical::canonical_pipeline;

#[must_use]
pub fn fastq_minimal_pipeline() -> PipelineSpec {
    let canonical = canonical_pipeline();
    PipelineSpec {
        stages: canonical.required,
    }
}
