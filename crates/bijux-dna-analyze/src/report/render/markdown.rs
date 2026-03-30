//! Owner: bijux-dna-analyze
//! Minimal Markdown renderer for report models.

use anyhow::Result;

use bijux_dna_infra::atomic_write_bytes;

use crate::report::model::ReportModel;

#[allow(dead_code)]
#[allow(clippy::unnecessary_wraps)]
pub fn render_report_markdown(model: &ReportModel) -> Result<String> {
    let report = &model.report;
    let mut lines = Vec::new();
    lines.push("# Bijux Run Report\n".to_string());
    lines.push(format!("- Run ID: `{}`", report.run_id));
    lines.push(format!("- Stages: {}", report.stages.len()));
    lines.push(format!("- Completeness: `{}`", report.completeness.status));
    if let Some(verdict) = &report.pipeline_verdict {
        lines.push(format!("- Pipeline Verdict: `{:?}`", verdict.verdict));
        if !verdict.reasons.is_empty() {
            lines.push("\n## Verdict Reasons".to_string());
            for reason in &verdict.reasons {
                lines.push(format!("- {reason}"));
            }
        }
    }
    lines.push("\n## Stage Summary".to_string());
    for stage in &report.stages {
        lines.push(format!(
            "- `{}` via `{}` ({})",
            stage.stage_id, stage.tool_id, stage.tool_version
        ));
    }
    Ok(lines.join("\n"))
}

#[allow(dead_code)]
pub fn write_report_markdown(path: &std::path::Path, model: &ReportModel) -> Result<()> {
    let rendered = render_report_markdown(model)?;
    atomic_write_bytes(path, rendered.as_bytes()).map_err(anyhow::Error::from)
}
