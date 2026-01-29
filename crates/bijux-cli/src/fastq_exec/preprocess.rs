use bijux_core::domain::PipelineSpec;
use bijux_stages::{fastq_default_pipeline, DefaultPipelineOptions};

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(args: &bijux_stages::args::BenchFastqPreprocessArgs) -> PipelineSpec {
    fastq_default_pipeline(DefaultPipelineOptions {
        paired: args.r2.is_some(),
        ..Default::default()
    })
}

pub use crate::fastq_exec::preprocess_exec::{bench_fastq_preprocess, fastq_preprocess_run};
