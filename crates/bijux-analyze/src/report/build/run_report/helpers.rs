use std::path::Path;

use bijux_core::{RetentionContextV1, RetentionDefinitionV1, RetentionReportV1, StageReportV1};

pub(super) fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

pub(super) fn report_path_for(reports: &serde_json::Value, key: &str) -> Option<String> {
    reports
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

pub(super) fn stage_report_fields(report: Option<&StageReportV1>) -> (String, String, String) {
    report.map_or_else(
        || (String::new(), String::new(), String::new()),
        |report| {
            (
                report.metrics_path.clone(),
                report.tool_invocation_path.clone(),
                report.effective_config_path.clone(),
            )
        },
    )
}

pub(super) fn retention_context_from_report(
    path: Option<&str>,
) -> Option<(RetentionContextV1, RetentionDefinitionV1)> {
    let report = path
        .and_then(|path| read_json_value(Path::new(path)))
        .and_then(|value| serde_json::from_value::<RetentionReportV1>(value).ok())?;
    let definition = report
        .retention
        .as_ref()
        .map_or_else(|| "unknown".to_string(), |ret| ret.definition.clone());
    let conditions = report
        .retention
        .as_ref()
        .map_or_else(|| report.condition.clone(), |ret| ret.conditions.clone());
    let context = RetentionContextV1 {
        stage_id: report.stage_id,
        tool_id: report.tool_id,
        definition,
        conditions,
    };
    let definition = RetentionDefinitionV1 {
        stage_id: context.stage_id.clone(),
        tool_id: context.tool_id.clone(),
        numerator: "reads_out,bases_out".to_string(),
        denominator: "reads_in,bases_in".to_string(),
        conditions: context.conditions.clone(),
    };
    Some((context, definition))
}

pub(super) fn banks_from_report(
    path: Option<&str>,
    fallback: serde_json::Value,
) -> serde_json::Value {
    path.and_then(|path| read_json_value(Path::new(path)))
        .and_then(|value| value.get("banks").cloned())
        .unwrap_or(fallback)
}
