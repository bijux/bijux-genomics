use bijux_engine::api::PipelineSpec;

use super::essential::essential_stages;

#[must_use]
pub fn preprocess_pipeline() -> PipelineSpec {
    PipelineSpec {
        stages: essential_stages().into_iter().map(str::to_string).collect(),
    }
}
