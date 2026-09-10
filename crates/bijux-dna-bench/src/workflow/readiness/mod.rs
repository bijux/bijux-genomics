//! Owner: bijux-dna-bench
//! Readiness report evaluation and presentation over typed evidence.

mod dashboard;
mod dashboard_markdown;

pub use dashboard::{
    evaluate_full_benchmark_dashboard, FullBenchmarkDashboardEvidence, FullBenchmarkDashboardPaths,
};
pub use dashboard_markdown::render_full_benchmark_dashboard_markdown;

mod tool_execution_modes;
pub use tool_execution_modes::evaluate_tool_execution_modes;

mod command_audit;
pub use command_audit::write_recorded_command_audit;

mod command_artifacts;
pub use command_artifacts::publish_rendered_command_artifacts;

mod slurm_scripts;
pub use slurm_scripts::validate_benchmark_local_slurm_script_bodies;
