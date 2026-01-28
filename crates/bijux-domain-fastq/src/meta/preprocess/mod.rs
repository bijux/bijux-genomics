pub mod exec;

use bijux_engine::api::PipelineSpec;

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    _args: &crate::stages::args::BenchFastqPreprocessArgs,
) -> PipelineSpec {
    crate::pipeline::preprocess::preprocess_pipeline()
}
