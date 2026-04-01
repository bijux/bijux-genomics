//! Owner: bijux-dna-bench
//! Comparison utilities for bench.

pub mod diff;
pub mod report;
pub mod stratify;

pub use diff::compare_summaries;
pub use report::{CompareReport, MetricDiff};
