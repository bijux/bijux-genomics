//! Benchmarking helpers for v1.

pub use bijux_analyze::compare::compare_runs_with_baseline;
pub use bijux_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};

pub use crate::bam_router::{bench_bam_pipeline, bench_bam_stage};
pub use crate::fastq_router::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre,
};
