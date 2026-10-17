use std::path::Path;

use anyhow::{ensure, Context as _, Result};

pub(super) fn require_canonical_sections(report_path: &Path, bundle_dir: &Path) -> Result<()> {
    let canonical: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
    let sections = canonical
        .get("sections")
        .and_then(serde_json::Value::as_object)
        .context("canonical constructed report sections")?;
    ensure!(
        sections.contains_key("qc") && sections.contains_key("pipeline_verdict"),
        "real constructed report must contain governed sections"
    );
    let local: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("report.json"))?)?;
    let html = std::fs::read_to_string(bundle_dir.join("index.html"))?;
    let marker = r#"<script id="report-json" type="application/json">"#;
    let (_, script) = html.split_once(marker).context("embedded canonical report script")?;
    let (script, _) = script.split_once("</script>").context("embedded report end")?;
    let embedded: serde_json::Value = serde_json::from_str(script)?;
    let mut failures = Vec::new();
    if local.get("sections") != canonical.get("sections") {
        failures.push("bundle-local sections".to_string());
    }
    for (name, expected) in sections {
        if embedded.get("sections").and_then(|value| value.get(name)) != Some(expected) {
            failures.push(format!("embedded section {name}"));
        }
    }
    eprintln!(
        "ANALYZE_REPORT_PROJECTION_CONTROL_PASS: canonical_keys={:?}; differences={failures:?}",
        sections.keys().collect::<Vec<_>>()
    );
    ensure!(failures.is_empty(), "constructed canonical report lost data: {failures:?}");
    Ok(())
}
