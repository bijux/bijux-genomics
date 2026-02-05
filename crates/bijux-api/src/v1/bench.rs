//! Benchmarking helpers for v1.
//!
//! Stability: v1 (stable).

pub use bijux_analyze::compare::compare_runs_with_baseline;
pub use bijux_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};

pub use crate::args::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};
pub use bijux_benchmark::{benchmark_runs, write_benchmark_exports};
pub use bijux_core::contract::{Objective, ObjectiveSpec, ObjectiveWeights};
pub use bijux_core::selection::objective_spec;
pub use bijux_planner_bam::stage_api::{bam_stage_completeness, BamStage};
pub use bijux_planner_fastq::stage_api as fastq_banks;
pub use bijux_planner_fastq::stage_api::args as fastq_args;
pub use bijux_planner_fastq::stage_api::banks as fastq_bank_ops;
pub use bijux_planner_fastq::stage_api::*;

pub use crate::handlers::bam::{bench_bam_pipeline, bench_bam_stage};
pub use crate::handlers::fastq::*;
