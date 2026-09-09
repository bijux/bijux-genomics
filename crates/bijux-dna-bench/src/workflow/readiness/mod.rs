//! Owner: bijux-dna-bench
//! Readiness report evaluation and presentation over typed evidence.

mod dashboard;
mod dashboard_markdown;

pub use dashboard::{
    evaluate_full_benchmark_dashboard, FullBenchmarkDashboardEvidence, FullBenchmarkDashboardPaths,
};
pub use dashboard_markdown::render_full_benchmark_dashboard_markdown;

mod command_audit;
pub use command_audit::write_recorded_command_audit;
