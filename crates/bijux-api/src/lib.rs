//! Public API entrypoints for planning and running bijux pipelines.

pub mod args;
pub mod bam_plan;
pub mod bam_router;
pub mod bam_support;
pub mod cross_router;
pub mod fastq_router;
pub mod fastq_stats_neutral;
pub mod run;

pub use args::{
    BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs, FastqCrossArgs, RunRequest, RunResult,
};
pub use bijux_stages_fastq::args as fastq_args;
pub use run::{run_pipeline, RunMode};
