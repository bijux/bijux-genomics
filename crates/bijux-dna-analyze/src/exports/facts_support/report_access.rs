use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_runtime::{FactsRowV1, StageReportV1};

use crate::model::JsonBlob;

pub(super) fn stage_report_path(reports: &JsonBlob) -> Option<String> {
    reports
        .as_value()
        .get("stage_report")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

pub(super) fn stage_report_for_row(row: &FactsRowV1) -> Option<StageReportV1> {
    let path = stage_report_path(&JsonBlob::from(row.reports.clone()))?;
    let report_raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&report_raw).ok()
}

pub(super) fn tool_invocation_for_stage(report: &StageReportV1) -> Option<ToolInvocationV1> {
    let raw = std::fs::read_to_string(&report.tool_invocation_path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub(super) fn stage_outputs_for_row(row: &FactsRowV1) -> Vec<String> {
    let Some(path) = stage_report_path(&JsonBlob::from(row.reports.clone())) else {
        return Vec::new();
    };
    let Ok(report_raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(report) = serde_json::from_str::<StageReportV1>(&report_raw) else {
        return Vec::new();
    };
    report.outputs
}
