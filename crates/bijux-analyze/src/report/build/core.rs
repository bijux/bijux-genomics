use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use super::bench::{bench_schema_json, print_bench_schema};
pub use super::bench::{derived_metrics_for_stage_json, rank_trim_tools};
pub use super::bench::{
    write_correct_report, write_filter_report, write_merge_report, write_qc_post_report,
    write_stats_report, write_trim_report, write_umi_report, write_validate_report,
};

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

use super::sections::schema::{
    build_report_sections, report_completeness, report_contract, report_metric_semantics,
};
use super::sections::{
    accounting_section, adapter_config_section, adapter_inference_section, assertions_section,
    bench_summary_section, comparison_view_section, decision_trace_section, failure_hints_section,
    filter_interpretation_section, findings_section, impact_metrics_section, params_excerpt,
    pipeline_verdict_from_rows, pipeline_verdict_section, qc_artifacts_section,
    qc_improvement_section, read_tool_invocation, report_path_for, reproducibility_section,
    scientific_provenance_section, stage_completeness_table, stage_confidence_section,
    stage_plots_section,
};
use crate::export::write_run_summary_json;
use crate::model::stable_sort_records;
use crate::model::JsonBlob;
use crate::report::model::ReportModel;
use crate::report::render::json::write_report_json;
use bijux_core::observability::FilterReportV1;
use bijux_core::{
    AssetsProvenanceV1, FactsRowV1, ReportProvenanceV1, ReportSchemaV1, ReportStageSummaryV1,
    RetentionContextV1, RetentionDefinitionV1, RetentionReportV1, StageReportV1, TelemetryEventV1,
};

/// Build a run report model from facts rows.
///
/// # Errors
/// Returns an error if report assembly fails.
#[allow(clippy::too_many_lines)]
pub fn build_run_report_model(base_dir: &Path, rows: &[FactsRowV1]) -> Result<ReportModel> {
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
    let mut missing_by_stage: std::collections::BTreeMap<String, (Vec<String>, Vec<String>)> =
        std::collections::BTreeMap::new();
    let mut metric_provenance: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();

    for row in &ordered {
        let stage_report_path = report_path_for(&row.reports, "stage_report");
        if stage_report_path.is_none() {
            missing_reports.push(format!("{}:stage_report", row.stage_id));
            missing_by_stage
                .entry(row.stage_id.clone())
                .or_default()
                .1
                .push("stage_report".to_string());
        }
        let stage_report = stage_report_path
            .as_deref()
            .and_then(|path| read_json_value(Path::new(path)))
            .and_then(|value| serde_json::from_value::<StageReportV1>(value).ok());

        let (metrics_path, tool_invocation_path, effective_config_path) =
            stage_report_fields(stage_report.as_ref());
        let tool_invocation_path_clone = tool_invocation_path.clone();
        if metrics_path.is_empty() {
            missing_reports.push(format!("{}:metrics_path", row.stage_id));
            missing_by_stage
                .entry(row.stage_id.clone())
                .or_default()
                .1
                .push("metrics_path".to_string());
        }
        if row.metrics == serde_json::json!({}) {
            missing_metrics.push(format!("{}:metrics", row.stage_id));
            missing_by_stage
                .entry(row.stage_id.clone())
                .or_default()
                .0
                .push("metrics".to_string());
        }

        let retention_report_path = report_path_for(&row.reports, "retention_report");
        if retention_report_path.is_none() && row.reads_in != row.reads_out {
            missing_reports.push(format!("{}:retention_report", row.stage_id));
            missing_by_stage
                .entry(row.stage_id.clone())
                .or_default()
                .1
                .push("retention_report".to_string());
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

        if !tool_invocation_path_clone.is_empty() {
            if let Some(invocation) = read_tool_invocation(Path::new(&tool_invocation_path_clone)) {
                let excerpt = params_excerpt(&invocation.parameters_json_normalized, 6);
                metric_provenance.insert(
                    row.stage_id.clone(),
                    serde_json::json!({
                        "tool_id": row.tool_id,
                        "params_hash": row.params_hash,
                        "normalized_params_excerpt": excerpt,
                    }),
                );
            }
        }
    }

    telemetry_events.sort();
    telemetry_events.dedup();
    let (telemetry_event_count, telemetry_error_count) = telemetry_counts(&telemetry_events);
    let telemetry_decisions = telemetry_decisions_from_paths(&telemetry_events);

    let metric_semantics = report_metric_semantics();
    let completeness = report_completeness(&missing_metrics, &missing_reports);
    let completeness_clone = completeness.clone();
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
        pipeline_verdict: Some(pipeline_verdict_from_rows(&ordered)),
        sections: serde_json::json!({}),
    };

    let mut sections: BTreeMap<String, JsonBlob> = build_report_sections(&report)
        .into_iter()
        .map(|(key, value)| (key, JsonBlob::new(value)))
        .collect();
    let mut model = ReportModel::empty(report);
    let stage_completeness = stage_completeness_table(&ordered, &missing_by_stage);
    let stage_confidence = stage_confidence_section(&ordered);
    sections.insert(
        "stage_completeness".to_string(),
        JsonBlob::new(stage_completeness.clone()),
    );
    sections.insert(
        "stage_confidence".to_string(),
        JsonBlob::new(stage_confidence.clone()),
    );
    sections.insert(
        "assertions".to_string(),
        JsonBlob::new(assertions_section(&ordered)),
    );
    sections.insert(
        "decision_trace".to_string(),
        JsonBlob::new(decision_trace_section(
            &ordered,
            &missing_by_stage,
            &telemetry_decisions,
        )),
    );
    sections.insert(
        "failure_hints".to_string(),
        JsonBlob::new(failure_hints_section(&ordered)),
    );
    sections.insert(
        "metric_provenance".to_string(),
        JsonBlob::new(serde_json::json!(metric_provenance)),
    );
    sections.insert(
        "bench_summary".to_string(),
        JsonBlob::new(bench_summary_section(base_dir)),
    );
    sections.insert(
        "pipeline_overview".to_string(),
        JsonBlob::new(pipeline_overview_section(&ordered)),
    );
    sections.insert(
        "key_findings".to_string(),
        JsonBlob::new(key_findings_section(
            &missing_metrics,
            &missing_reports,
            &ordered,
            &stage_confidence,
        )),
    );
    sections.insert(
        "stage_plots".to_string(),
        JsonBlob::new(stage_plots_section(&ordered)),
    );
    sections.insert(
        "accounting".to_string(),
        JsonBlob::new(accounting_section(&ordered)),
    );
    sections.insert(
        "impact_metrics".to_string(),
        JsonBlob::new(impact_metrics_section(&ordered)),
    );
    sections.insert(
        "findings".to_string(),
        JsonBlob::new(findings_section(&ordered)),
    );
    sections.insert(
        "reproducibility".to_string(),
        JsonBlob::new(reproducibility_section(&ordered, &telemetry_events)),
    );
    sections.insert(
        "pipeline_verdict".to_string(),
        JsonBlob::new(pipeline_verdict_section(&ordered)),
    );
    sections.insert(
        "scientific_provenance".to_string(),
        JsonBlob::new(scientific_provenance_section(&ordered)),
    );
    sections.insert(
        "data_contract_validation".to_string(),
        JsonBlob::new(data_contract_validation_section(&completeness_clone)),
    );
    sections.insert(
        "qc_delta".to_string(),
        JsonBlob::new(qc_delta_section(&ordered)),
    );
    sections.insert(
        "qc_artifacts".to_string(),
        JsonBlob::new(qc_artifacts_section(&ordered)),
    );
    sections.insert(
        "contaminant_summary".to_string(),
        JsonBlob::new(contaminant_summary_section(&ordered)),
    );
    sections.insert(
        "comparison_view".to_string(),
        JsonBlob::new(comparison_view_section(&ordered)),
    );
    sections.insert(
        "adapter_config".to_string(),
        JsonBlob::new(adapter_config_section(&ordered)),
    );
    model.sections = sections;
    model.tables.insert(
        "stage_completeness".to_string(),
        JsonBlob::new(stage_completeness),
    );
    Ok(model)
}

/// Write a run-level report from facts rows.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_run_report_from_facts(base_dir: &Path, rows: &[FactsRowV1]) -> Result<PathBuf> {
    std::fs::create_dir_all(base_dir)?;
    let path = base_dir.join("report.json");
    let model = build_run_report_model(base_dir, rows)?;
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

fn pipeline_overview_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let stages: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let (tier, rationale) = bijux_core::tool_tier_for(&row.stage_id, &row.tool_id);
            serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "tool_version": row.tool_version,
                "tool_tier": format!("{tier:?}").to_lowercase(),
                "tier_rationale": rationale,
                "scientific_preset": row.reports.get("scientific_preset").cloned().unwrap_or(serde_json::Value::Null),
                "params_hash": row.params_hash,
                "image_digest": row.image_digest,
                "input_hash": row.input_hash,
                "output_hashes": row.output_hashes,
            })
        })
        .collect();
    serde_json::json!({
        "stages": stages,
    })
}

fn key_findings_section(
    missing_metrics: &[String],
    missing_reports: &[String],
    rows: &[FactsRowV1],
    stage_confidence: &serde_json::Value,
) -> serde_json::Value {
    let mut findings = Vec::new();
    if !missing_metrics.is_empty() {
        findings.push(serde_json::json!({
            "kind": "missing_metrics",
            "count": missing_metrics.len(),
            "items": missing_metrics,
        }));
    }
    if !missing_reports.is_empty() {
        findings.push(serde_json::json!({
            "kind": "missing_reports",
            "count": missing_reports.len(),
            "items": missing_reports,
        }));
    }
    let failed: Vec<String> = rows
        .iter()
        .filter(|row| row.exit_code != 0)
        .map(|row| format!("{}:{}", row.stage_id, row.tool_id))
        .collect();
    if !failed.is_empty() {
        findings.push(serde_json::json!({
            "kind": "tool_failures",
            "count": failed.len(),
            "items": failed,
        }));
    }
    if let Some(entries) = stage_confidence
        .get("entries")
        .and_then(serde_json::Value::as_array)
    {
        let fragile: Vec<serde_json::Value> = entries
            .iter()
            .filter(|entry| {
                entry
                    .get("score")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(1.0)
                    < 0.6
            })
            .cloned()
            .collect();
        if !fragile.is_empty() {
            findings.push(serde_json::json!({
                "kind": "low_confidence_stages",
                "count": fragile.len(),
                "items": fragile,
            }));
        }
    }
    serde_json::Value::Array(findings)
}

fn data_contract_validation_section(
    completeness: &bijux_core::ReportCompletenessV1,
) -> serde_json::Value {
    serde_json::json!({
        "status": completeness.status,
        "missing_metrics": completeness.missing_metrics,
        "missing_reports": completeness.missing_reports,
    })
}

