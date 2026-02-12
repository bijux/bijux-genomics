//! HTML rendering subsystem.

mod sections;
mod template;

use crate::report::model::ReportModel;
use anyhow::Result;

use sections::{build_nav_items, build_section_blocks, build_stage_panels, build_stage_tabs};
use template::build_html_template;

#[allow(dead_code)]
pub fn render_report_html(model: &ReportModel) -> Result<String> {
    let report = &model.report;
    let mut report_json_value = serde_json::to_value(report)?;
    if let Some(root) = report_json_value.as_object_mut() {
        root.insert(
            "bundle_schema_version".to_string(),
            serde_json::Value::String("bijux.report_bundle.v1".to_string()),
        );
        if let Some(sections) = root
            .entry("sections")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
        {
            sections
                .entry("fastq".to_string())
                .or_insert_with(|| serde_json::json!({
                    "schema_version": "bijux.report.section.fastq.v1",
                    "stages": report.stages.len(),
                }));
            if let Some(run_provenance) = sections
                .entry("run_provenance".to_string())
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
            {
                run_provenance
                    .entry("manifest_signature_sha256".to_string())
                    .or_insert_with(|| serde_json::Value::String("unknown".to_string()));
            }
        }
    }
    let report_json = serde_json::to_string_pretty(&report_json_value)?;
    let sections = report.sections.as_object().cloned().unwrap_or_default();
    let mut section_keys: Vec<String> = sections.keys().cloned().collect();
    section_keys.sort();

    let nav_items = build_nav_items(&section_keys);
    let section_blocks = build_section_blocks(&sections, &section_keys)?;
    let mut stages = report.stages.clone();
    stages.sort_by(|left, right| match left.stage_id.cmp(&right.stage_id) {
        std::cmp::Ordering::Equal => left.tool_id.cmp(&right.tool_id),
        ordering => ordering,
    });
    let stage_tabs = build_stage_tabs(&stages);
    let stage_panels = build_stage_panels(&stages);

    let stage_plots = sections
        .get("stage_plots")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let stage_plots_json = serde_json::to_string(&stage_plots)?;
    let reproducibility = sections
        .get("reproducibility")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let repro_json = serde_json::to_string_pretty(&reproducibility)?;
    let command = reproducibility
        .get("command")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");

    Ok(build_html_template(
        &report_json,
        &nav_items,
        &section_blocks,
        &stage_tabs,
        &stage_panels,
        &stage_plots_json,
        &repro_json,
        command,
    ))
}

#[allow(dead_code)]
pub fn write_report_html(path: &std::path::Path, model: &ReportModel) -> Result<()> {
    let rendered = render_report_html(model)?;
    bijux_dna_infra::atomic_write_bytes(path, rendered.as_bytes()).map_err(anyhow::Error::from)
}
