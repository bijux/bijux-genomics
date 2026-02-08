pub mod bench;
pub mod bench_results_fastq;
pub mod core_other;
pub mod core_trim;
pub mod quality;
pub mod query_shared;
pub mod reports;

#[allow(dead_code)]
pub const QUERY_MODULES: &[&str] = &[
    "bench",
    "bench_results_fastq",
    "query_shared",
    "core_other",
    "core_trim",
    "quality",
    "reports",
];
