use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::run_report_schema::{
    build_report_sections, report_completeness, report_contract, report_metric_semantics,
};
use super::run_report_sections::{
    adapter_inference_section, filter_interpretation_section, qc_improvement_section,
};
use crate::export::write_run_summary_json;
use crate::model::stable_sort_records;
use crate::model::JsonBlob;
use crate::report::model::ReportModel;
use crate::report::render_json::write_report_json;
use bijux_core::{
    AssetsProvenanceV1, FactsRowV1, ReportProvenanceV1, ReportSchemaV1, ReportStageSummaryV1,
    RetentionContextV1, RetentionDefinitionV1, RetentionReportV1, StageReportV1, TelemetryEventV1,
};

/// Build a run report model from facts rows.
///
/// # Errors
/// Returns an error if report assembly fails.
#[allow(clippy::too_many_lines)]
pub fn build_run_report_model(rows: &[FactsRowV1]) -> Result<ReportModel> {
    let mut ordered = rows.to_vec();
    stable_sort_records(&mut ordered, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            row.input_hash.as_str(),
        )
    });
    let run_id = ordered
        .first()
        .map_or_else(String::new, |row| row.run_id.clone());
    let mut stages = Vec::new();
    let mut provenance = Vec::new();
    let mut retention_context = Vec::new();
    let mut retention_definition = Vec::new();
    let mut assets_provenance = Vec::new();
    let mut telemetry_events = Vec::new();
    let mut missing_metrics = Vec::new();
    let mut missing_reports = Vec::new();

    for row in &ordered {
        let stage_report_path = report_path_for(&row.reports, "stage_report");
        if stage_report_path.is_none() {
            missing_reports.push(format!("{}:stage_report", row.stage_id));
        }
        let stage_report = stage_report_path
            .as_deref()
            .and_then(|path| read_json_value(Path::new(path)))
            .and_then(|value| serde_json::from_value::<StageReportV1>(value).ok());

        let (metrics_path, tool_invocation_path, effective_config_path) =
            stage_report_fields(stage_report.as_ref());
        if metrics_path.is_empty() {
            missing_reports.push(format!("{}:metrics_path", row.stage_id));
        }
        if row.metrics == serde_json::json!({}) {
            missing_metrics.push(format!("{}:metrics", row.stage_id));
        }

        let retention_report_path = report_path_for(&row.reports, "retention_report");
        if retention_report_path.is_none() && row.reads_in != row.reads_out {
            missing_reports.push(format!("{}:retention_report", row.stage_id));
        }
        if let Some((context, definition)) =
            retention_context_from_report(retention_report_path.as_deref())
        {
            retention_context.push(context);
            retention_definition.push(definition);
        }

        let bank_report_path = report_path_for(&row.reports, "bank_report");
        let banks_value = banks_from_report(bank_report_path.as_deref(), row.bank_hashes.clone());
        assets_provenance.push(AssetsProvenanceV1 {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            banks: banks_value,
        });

        if let Some(path) = telemetry_path_from_stage_report(stage_report_path.as_deref()) {
            telemetry_events.push(path);
        }

        stages.push(ReportStageSummaryV1 {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            params_hash: row.params_hash.clone(),
            input_hash: row.input_hash.clone(),
            runtime_s: row.runtime_s,
            memory_mb: row.memory_mb,
            exit_code: row.exit_code,
            metrics_path,
            tool_invocation_path,
            effective_config_path,
            stage_report_path: stage_report_path
                .as_deref()
                .map_or_else(String::new, ToString::to_string),
            retention_report_path,
            bank_report_path,
        });

        provenance.push(ReportProvenanceV1 {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            image_digest: row
                .image_digest
                .clone()
                .or_else(|| Some("unknown".to_string())),
            trace_id: row.trace_id.clone(),
            span_id: row.span_id.clone(),
            params_hash: row.params_hash.clone(),
            bank_hashes: row.bank_hashes.clone(),
        });
    }

    telemetry_events.sort();
    telemetry_events.dedup();
    let (telemetry_event_count, telemetry_error_count) = telemetry_counts(&telemetry_events);

    let metric_semantics = report_metric_semantics();
    let completeness = report_completeness(&missing_metrics, &missing_reports);
    let qc_improvement = qc_improvement_section(&ordered);
    let filter_interpretation = filter_interpretation_section(&ordered);
    let adapter_inference = adapter_inference_section(&ordered);
    let final_qc_summary = serde_json::json!({
        "qc": qc_improvement.clone(),
        "adapter_inference": adapter_inference.clone(),
    });
    let report = ReportSchemaV1 {
        schema_version: "bijux.report.v1".to_string(),
        contract: report_contract(),
        run_id,
        completeness,
        stages,
        provenance,
        retention_definition,
        retention_context,
        assets_provenance,
        metric_semantics,
        telemetry: serde_json::json!({
            "events": telemetry_events,
            "event_count": telemetry_event_count,
            "error_count": telemetry_error_count,
        }),
        qc_improvement,
        final_qc_summary,
        filter_interpretation,
        adapter_inference,
        sections: serde_json::json!({}),
    };

    let sections = build_report_sections(&report)
        .into_iter()
        .map(|(key, value)| (key, JsonBlob::new(value)))
        .collect();
    let mut model = ReportModel::empty(report);
    model.sections = sections;
    Ok(model)
}

/// Write a run-level report from facts rows.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_run_report_from_facts(base_dir: &Path, rows: &[FactsRowV1]) -> Result<PathBuf> {
    let path = base_dir.join("report.json");
    let model = build_run_report_model(rows)?;
    write_report_json(&path, &model).context("write report.json")?;
    Ok(path)
}

/// Write a deterministic run summary JSON from facts rows.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_run_summary_from_facts(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    write_run_summary_json(path, rows)
}

fn read_json_value(path: &Path) -> Option<serde_json::Value> {
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

fn stage_report_fields(report: Option<&StageReportV1>) -> (String, String, String) {
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

fn retention_context_from_report(
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

fn banks_from_report(path: Option<&str>, fallback: serde_json::Value) -> serde_json::Value {
    path.and_then(|path| read_json_value(Path::new(path)))
        .and_then(|value| value.get("banks").cloned())
        .unwrap_or(fallback)
}

fn telemetry_path_from_stage_report(path: Option<&str>) -> Option<String> {
    path.and_then(|path| {
        Path::new(path).parent().map(|parent| {
            parent
                .join("telemetry")
                .join("events.jsonl")
                .display()
                .to_string()
        })
    })
}

fn telemetry_counts(paths: &[String]) -> (usize, usize) {
    let mut total_events = 0usize;
    let mut error_events = 0usize;
    for path in paths {
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            total_events += 1;
            if let Ok(event) = serde_json::from_str::<TelemetryEventV1>(line) {
                if event.event_name == "error" || event.status == "error" {
                    error_events += 1;
                }
            }
        }
    }
    (total_events, error_events)
}
