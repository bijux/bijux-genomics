//! Owner: bijux-analyze
//! Minimal HTML renderer for report models.

use anyhow::Result;

use crate::report::model::ReportModel;

#[allow(dead_code)]
pub fn render_report_html(model: &ReportModel) -> Result<String> {
    let json = serde_json::to_string_pretty(&model.report)?;
    Ok(format!(
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
  <pre>{json}</pre>
</body>
</html>"#
    ))
}

#[allow(dead_code)]
pub fn write_report_html(path: &std::path::Path, model: &ReportModel) -> Result<()> {
    std::fs::write(path, render_report_html(model)?).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::render_report_html;
    use crate::report::model::ReportModel;
    use bijux_core::ReportSchemaV1;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn report_html_snapshot() -> anyhow::Result<()> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let report_path = manifest_dir
            .join("tests")
            .join("snapshots")
            .join("run_report.json");
        let raw = fs::read_to_string(&report_path)?;
        let report: ReportSchemaV1 = serde_json::from_str(&raw)?;
        let model = ReportModel::empty(report);
        let html = render_report_html(&model)?;
        let snapshot_path = manifest_dir
            .join("tests")
            .join("snapshots")
            .join("run_report.html");
        let expected = fs::read_to_string(&snapshot_path)?;
        let extract_json = |doc: &str| -> anyhow::Result<serde_json::Value> {
            let start = doc
                .find("<pre>")
                .ok_or_else(|| anyhow::anyhow!("missing <pre>"))?
                + "<pre>".len();
            let end = doc
                .find("</pre>")
                .ok_or_else(|| anyhow::anyhow!("missing </pre>"))?;
            let json_raw = &doc[start..end];
            Ok(serde_json::from_str(json_raw)?)
        };
        let expected_json = extract_json(&expected)?;
        let actual_json = extract_json(&html)?;
        assert_eq!(expected_json, actual_json);
        Ok(())
    }
}
