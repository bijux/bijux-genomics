mod correct;
mod filter;
mod merge;
mod qc_post;
mod screen;
mod stats_neutral;
mod trim;
mod umi;
mod validate_pre;

pub use correct::bench_fastq_correct;
pub use filter::bench_fastq_filter;
pub use merge::bench_fastq_merge;
pub use qc_post::bench_fastq_qc_post;
pub use screen::bench_fastq_screen;
pub use stats_neutral::bench_fastq_stats_neutral;
pub use trim::bench_fastq_trim;
pub use umi::bench_fastq_umi;
pub use validate_pre::bench_fastq_validate_pre;
