use bijux_engine::api::PipelineSpec;

use crate::pipeline::{fastq_default_pipeline, DefaultPipelineOptions};

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(args: &crate::stages::args::BenchFastqPreprocessArgs) -> PipelineSpec {
    fastq_default_pipeline(DefaultPipelineOptions {
        paired: args.r2.is_some(),
        ..Default::default()
    })
}

pub use crate::stages::preprocess_exec::{bench_fastq_preprocess, fastq_preprocess_run};
