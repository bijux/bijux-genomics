//! Reporting and analysis helpers for v1.
//!
//! Stability: v1 (stable).

mod html_bundle;

pub use crate::runtime::run::render_report;
pub use crate::surface::request_contracts::{RenderReportRequest, RenderReportResult};

pub use bijux_dna_analyze::exports::write_stage_summary_csv;
pub use bijux_dna_analyze::*;
pub use html_bundle::render_report_bundle_html;
