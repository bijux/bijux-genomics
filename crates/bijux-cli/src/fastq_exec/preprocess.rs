/// Build the preprocess pipeline plan.
#[must_use]
pub fn fastq_preprocess_plan(
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
) -> bijux_core::domain::PipelineSpec {
    crate::fastq_router::fastq_preprocess_plan(args)
}

pub use crate::fastq_exec::preprocess_exec::{bench_fastq_preprocess, fastq_preprocess_run};
