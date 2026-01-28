use bijux_engine::api::PipelineSpec;

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    _args: &crate::stages::args::BenchFastqPreprocessArgs,
) -> PipelineSpec {
    crate::contracts::pipeline_contract::preprocess_pipeline()
}

pub use crate::stages::preprocess_exec::{bench_fastq_preprocess, fastq_preprocess_run};
