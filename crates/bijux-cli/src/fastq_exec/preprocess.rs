use bijux_core::domain::PipelineSpec;
use bijux_stages_fastq::fastq::preprocess::plan_preprocess;

/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> PipelineSpec {
    plan_preprocess(args).pipeline
}

pub use crate::fastq_exec::preprocess_exec::{bench_fastq_preprocess, fastq_preprocess_run};
