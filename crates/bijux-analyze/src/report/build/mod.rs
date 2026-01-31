mod bench_helpers;
mod bench_reports;
mod bench_schema;
mod run_report;

pub use bench_helpers::{derived_metrics_for_stage_json, rank_trim_tools};
pub use bench_reports::{
    write_correct_report, write_filter_report, write_merge_report, write_qc_post_report,
    write_stats_report, write_trim_report, write_umi_report, write_validate_report,
};
pub use bench_schema::{bench_schema_json, print_bench_schema};
pub use run_report::{write_run_report_from_facts, write_run_summary_from_facts};

#[cfg(test)]
mod tests;
