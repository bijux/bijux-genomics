//! Owner: bijux-analyze
//! Render report bundle (HTML + JSON + assets).

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::model::ReportModel;
use crate::report::render::html::render_report_html;

pub fn write_report_bundle(dir: &Path, model: &ReportModel) -> Result<()> {
    std::fs::create_dir_all(dir).context("create report bundle dir")?;
    let html = render_report_html(model)?;
    let index_path = dir.join("index.html");
    std::fs::write(&index_path, html).context("write report bundle index.html")?;
    let report_json = serde_json::to_vec_pretty(&model.report)?;
    let report_path = dir.join("report.json");
    std::fs::write(&report_path, report_json).context("write report bundle report.json")?;
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&assets_dir).context("create report assets dir")?;
    let style_path = assets_dir.join("style.css");
    std::fs::write(
        &style_path,
        "body{font-family:system-ui,-apple-system,sans-serif;margin:2rem;background:#f7f7f9;color:#111}pre{padding:1rem;background:#fff;border-radius:8px;overflow:auto;box-shadow:0 1px 4px rgba(0,0,0,0.08)}",
    )
    .context("write report bundle css")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_report_bundle;
    use crate::report::model::ReportModel;
    use bijux_core::ReportSchemaV1;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn report_bundle_snapshot() -> anyhow::Result<()> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let report_path = manifest_dir
            .join("tests")
            .join("snapshots")
            .join("run_report.json");
        let raw = fs::read_to_string(&report_path)?;
        let report: ReportSchemaV1 = serde_json::from_str(&raw)?;
        let model = ReportModel::empty(report);
        let tmp = tempfile::tempdir()?;
        let bundle_dir = tmp.path().join("bundle");
        write_report_bundle(&bundle_dir, &model)?;
        let index_path = bundle_dir.join("index.html");
        let actual = fs::read_to_string(index_path)?;
        let snapshot_path = manifest_dir
            .join("tests")
            .join("snapshots")
            .join("run_report_bundle_index.html");
        let expected = fs::read_to_string(snapshot_path)?;
        let extract_json = |doc: &str| -> anyhow::Result<serde_json::Value> {
            let marker = r#"<script id="report-json" type="application/json">"#;
            let start = doc
                .find(marker)
                .ok_or_else(|| anyhow::anyhow!("missing report-json script"))?
                + marker.len();
            let end = doc
                .find("</script>")
                .ok_or_else(|| anyhow::anyhow!("missing </script>"))?;
            let json_raw = &doc[start..end];
            Ok(serde_json::from_str(json_raw)?)
        };
        let expected_json = extract_json(&expected)?;
        let actual_json = extract_json(&actual)?;
        assert_eq!(expected_json, actual_json);
        Ok(())
    }
}
