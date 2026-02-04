//! Reporting and analysis helpers for v1.

pub use crate::args::{RenderReportRequest, RenderReportResult};
pub use crate::run::render_report;

pub use bijux_analyze::export::write_stage_summary_csv;
pub use bijux_analyze::{
    load_facts_auto, load_run_summary, write_correct_report, write_filter_report,
    write_merge_report, write_qc_post_report, write_run_report_from_facts,
    write_run_summary_from_facts, write_stats_report, write_trim_report, write_umi_report,
    write_validate_report,
};
