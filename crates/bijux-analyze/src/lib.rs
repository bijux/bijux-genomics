pub mod aggregate;
pub mod explain;
pub mod load;
pub mod render;

pub mod compare;
pub mod facts;
pub mod failure;
pub mod ranking;
pub mod report;
pub mod semantic;

pub use aggregate::*;
pub use load::*;

pub use bijux_core::metrics::{MetricEnvelope, MetricSet};
pub use compare::{compare_runs, RunComparison};
pub use failure::{classify_raw_failure, BenchmarkFailure, FailureClass};
pub use ranking::{build_rankings, print_rank_explain, RankInput, RankingEntry, RankingMode};
pub use report::{
    print_bench_schema, write_correct_report, write_filter_report, write_merge_report,
    write_qc_post_report, write_run_report_from_facts, write_run_summary_from_facts,
    write_stats_report, write_trim_report, write_umi_report, write_validate_report,
};
pub use semantic::{
    semantic_filter, semantic_stats, semantic_trim, semantic_validate, ContaminationMetrics,
    IntegrityMetrics, MetricDescriptor, QualityShiftMetrics, RetentionMetrics, SemanticMetrics,
};
