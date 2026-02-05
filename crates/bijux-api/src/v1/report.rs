//! Reporting and analysis helpers for v1.
//!
//! Stability: v1 (stable).

pub use crate::args::{RenderReportRequest, RenderReportResult};
pub use crate::run::render_report;

pub use bijux_analyze::export::write_stage_summary_csv;
pub use bijux_analyze::*;

#[must_use]
pub fn render_report_bundle_html(report: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>bijux analyze report</title>
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      margin: 2rem;
      line-height: 1.4;
      background: #f7f7f9;
      color: #111;
    }}
    pre {{
      padding: 1rem;
      background: #fff;
      border-radius: 8px;
      overflow: auto;
      box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    }}
  </style>
</head>
<body>
  <h1>bijux analyze report</h1>
  <pre>{pretty}</pre>
</body>
</html>"#
    )
}
