use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_runtime::{FactsRowV1, StageReportV1};

use crate::model::{FactsSummary, JsonBlob};

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

pub(super) fn params_excerpt(value: &serde_json::Value, limit: usize) -> serde_json::Value {
    let Some(obj) = value.as_object() else {
        return value.clone();
    };
    let mut keys: Vec<_> = obj.keys().cloned().collect();
    keys.sort();
    let mut out = serde_json::Map::new();
    for key in keys.into_iter().take(limit) {
        if let Some(v) = obj.get(&key) {
            out.insert(key, v.clone());
        }
    }
    serde_json::Value::Object(out)
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

#[must_use]
pub fn summarize_facts(rows: &[FactsRowV1]) -> FactsSummary {
    let stages = rows.len();
    let mut run_ids = std::collections::BTreeSet::new();
    let mut total_runtime_s = 0.0;
    for row in rows {
        run_ids.insert(row.run_id.clone());
        total_runtime_s += row.runtime_s;
    }
    let runs = run_ids.len();
    let avg_runtime_s = if stages == 0 {
        0.0
    } else {
        let denom = f64::from(u32::try_from(stages).unwrap_or(u32::MAX));
        total_runtime_s / denom
    };
    FactsSummary {
        runs,
        stages,
        total_runtime_s,
        avg_runtime_s,
    }
}
