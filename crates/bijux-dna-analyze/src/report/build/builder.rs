use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::contracts::{
    analysis_selection_contract_section, enforce_report_completeness_contract,
    pipeline_defaults_section, required_vcf_metric_keys,
};
use super::provenance::{
    cross_domain_handoff_section, normalize_report_path, run_provenance_section,
};
use super::report_inputs::{
    banks_from_report, contaminant_summary_section, qc_delta_section, read_json_value,
    retention_context_from_report, stage_report_fields, telemetry_counts,
    telemetry_decisions_from_paths, telemetry_path_from_stage_report,
    telemetry_timeline_from_paths,
};
use super::report_sections::{
    data_contract_validation_section, key_findings_section, pipeline_overview_section,
};
pub use crate::report::bench::{bench_schema_json, print_bench_schema};
pub use crate::report::bench::{derived_metrics_for_stage_json, rank_trim_tools};
pub use crate::report::bench::{
    write_correct_report, write_filter_report, write_merge_report, write_qc_post_report,
    write_screen_report, write_stats_report, write_trim_polyg_report, write_trim_report,
    write_trim_terminal_damage_report, write_umi_report, write_validate_report,
};

use crate::exports::write_run_summary_json;
use crate::model::stable_sort_records;
use crate::model::JsonBlob;
use crate::report::render::json::write_report_json;
use crate::report::render_model::ReportModel;
use crate::report::sections::schema::{
    build_report_sections, report_completeness, report_contract, report_metric_semantics,
};
use crate::report::sections::{
    accounting_section, adapter_config_section, adapter_inference_section, assertions_section,
    bam_accounting_section, bam_findings_section, bam_plots_section, bam_verdict_table,
    bench_summary_section, claims_registry_section, comparison_view_section,
    decision_trace_section, failure_hints_section, filter_interpretation_section, findings_section,
    impact_metrics_section, params_excerpt, pipeline_verdict_from_rows, pipeline_verdict_section,
    qc_artifacts_section, qc_improvement_section, read_tool_invocation, report_path_for,
    reproducibility_section, scientific_provenance_section, stage_completeness_table,
    stage_confidence_section, stage_plots_section,
};
use bijux_dna_domain_bam::prelude::STAGE_PREFIX as BAM_STAGE_PREFIX;
use bijux_dna_domain_fastq::prelude::STAGE_PREFIX as FASTQ_STAGE_PREFIX;
use bijux_dna_runtime::{
    AssetsProvenanceV1, FactsRowV1, ReportProvenanceV1, ReportSchemaV1, ReportStageSummaryV1,
    StageReportV1,
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
    let run_id = ordered.first().map_or_else(String::new, |row| row.run_id.clone());
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
        if row.tool_version.trim().is_empty() || row.params_hash.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "report policy violation: stage {} missing tool_version or params_hash",
                row.stage_id
            ));
        }
        let metric_provenance_contract = row.effective_metric_provenance();
        if !metric_provenance_contract.is_complete() {
            return Err(anyhow::anyhow!(
                "report policy violation: stage {} has incomplete metric provenance",
                row.stage_id
            ));
        }
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
        let metrics_path = normalize_report_path(base_dir, &metrics_path);
        let tool_invocation_path = normalize_report_path(base_dir, &tool_invocation_path);
        let effective_config_path = normalize_report_path(base_dir, &effective_config_path);
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
            missing_by_stage.entry(row.stage_id.clone()).or_default().0.push("metrics".to_string());
        }
        for key in required_vcf_metric_keys(&row.stage_id) {
            if row.metrics.get(*key).is_none() {
                missing_metrics.push(format!("{}:{key}", row.stage_id));
                missing_by_stage
                    .entry(row.stage_id.clone())
                    .or_default()
                    .0
                    .push((*key).to_string());
            }
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
            let normalized = normalize_report_path(base_dir, &path);
            if !normalized.is_empty() {
                telemetry_events.push(normalized);
            }
        }

        let stage_report_path = stage_report_path
            .as_deref()
            .map(|path| normalize_report_path(base_dir, path))
            .unwrap_or_default();
        let retention_report_path =
            retention_report_path.as_deref().map(|path| normalize_report_path(base_dir, path));
        let bank_report_path =
            bank_report_path.as_deref().map(|path| normalize_report_path(base_dir, path));
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
            stage_report_path,
            retention_report_path,
            bank_report_path,
        });

        provenance.push(ReportProvenanceV1 {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            image_digest: row.image_digest.clone().or_else(|| Some("unknown".to_string())),
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
                        "metric_provenance": metric_provenance_contract,
                        "tool_id": row.tool_id,
                        "params_hash": row.params_hash,
                        "normalized_params_excerpt": excerpt,
                    }),
                );
            }
        }
    }
    let missing_vcf_required = missing_metrics
        .iter()
        .filter(|entry| {
            entry.starts_with("vcf.")
                && !entry.ends_with(":metrics")
                && !entry.ends_with(":stage_report")
                && !entry.ends_with(":metrics_path")
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing_vcf_required.is_empty() {
        return Err(anyhow::anyhow!(
            "vcf downstream report contract violation: missing required metrics [{}]",
            missing_vcf_required.join(", ")
        ));
    }

    telemetry_events.sort();
    telemetry_events.dedup();
    let (telemetry_event_count, telemetry_error_count) = telemetry_counts(&telemetry_events);
    let telemetry_decisions = telemetry_decisions_from_paths(&telemetry_events);
    let telemetry_timeline = telemetry_timeline_from_paths(&telemetry_events);

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
            "timeline": telemetry_timeline,
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
    sections.insert("stage_completeness".to_string(), JsonBlob::new(stage_completeness.clone()));
    sections.insert("stage_confidence".to_string(), JsonBlob::new(stage_confidence.clone()));
    sections.insert("assertions".to_string(), JsonBlob::new(assertions_section(&ordered)));
    sections.insert(
        "decision_trace".to_string(),
        JsonBlob::new(decision_trace_section(&ordered, &missing_by_stage, &telemetry_decisions)),
    );
    sections.insert(
        "analysis_selection_contract".to_string(),
        JsonBlob::new(analysis_selection_contract_section()),
    );
    sections.insert("failure_hints".to_string(), JsonBlob::new(failure_hints_section(&ordered)));
    sections.insert(
        "metric_provenance".to_string(),
        JsonBlob::new(serde_json::json!(metric_provenance)),
    );
    sections.insert("bench_summary".to_string(), JsonBlob::new(bench_summary_section(base_dir)));
    sections.insert(
        "pipeline_overview".to_string(),
        JsonBlob::new(pipeline_overview_section(&ordered)),
    );
    sections.insert(
        "pipeline_defaults".to_string(),
        JsonBlob::new(pipeline_defaults_section(base_dir)?),
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
    sections.insert("stage_plots".to_string(), JsonBlob::new(stage_plots_section(&ordered)));
    sections.insert("accounting".to_string(), JsonBlob::new(accounting_section(&ordered)));
    if ordered.iter().any(|row| row.stage_id.starts_with(BAM_STAGE_PREFIX)) {
        sections.insert(
            "bam".to_string(),
            JsonBlob::new(serde_json::json!({
                "schema_version": "bijux.report.section.bam.v1",
                "stages": ordered.iter().filter(|row| row.stage_id.starts_with(BAM_STAGE_PREFIX)).count(),
            })),
        );
        sections
            .insert("bam_accounting".to_string(), JsonBlob::new(bam_accounting_section(&ordered)));
        sections.insert("bam_findings".to_string(), JsonBlob::new(bam_findings_section(&ordered)));
        sections.insert("bam_verdicts".to_string(), JsonBlob::new(bam_verdict_table(&ordered)));
        sections.insert("bam_plots".to_string(), JsonBlob::new(bam_plots_section(&ordered)));
    }
    if ordered.iter().any(|row| row.stage_id.starts_with("vcf.")) {
        let vcf_rows = ordered.iter().filter(|row| row.stage_id == "vcf.stats").collect::<Vec<_>>();
        let mut variants_total = 0_u64;
        let mut snps = 0_u64;
        let mut indels = 0_u64;
        let mut ti_tv = None::<f64>;
        let mut sample_name = "sample".to_string();
        let mut filter_breakdown = serde_json::json!({});
        let mut depth_distribution = serde_json::json!({});
        let mut call_summary = serde_json::json!({});
        let mut filter_summary = serde_json::json!({});
        for row in vcf_rows {
            if let Some(value) =
                row.metrics.get("variants_total").and_then(serde_json::Value::as_u64)
            {
                variants_total = value;
            }
            if let Some(value) = row.metrics.get("snps").and_then(serde_json::Value::as_u64) {
                snps = value;
            }
            if let Some(value) = row.metrics.get("indels").and_then(serde_json::Value::as_u64) {
                indels = value;
            }
            if let Some(value) = row.metrics.get("ti_tv").and_then(serde_json::Value::as_f64) {
                ti_tv = Some(value);
            }
            if let Some(value) = row.metrics.get("sample_name").and_then(serde_json::Value::as_str)
            {
                sample_name = value.to_string();
            }
            if let Some(value) = row.metrics.get("filter_breakdown") {
                filter_breakdown = value.clone();
            }
            if let Some(value) = row.metrics.get("depth_distribution") {
                depth_distribution = value.clone();
            }
            if let Some(value) = row.metrics.get("call_summary") {
                call_summary = value.clone();
            }
            if let Some(value) = row.metrics.get("filter_summary") {
                filter_summary = value.clone();
            }
        }
        sections.insert(
            "vcf".to_string(),
            JsonBlob::new(serde_json::json!({
                "schema_version": "bijux.vcf.stats.v1",
                "sample_name": sample_name,
                "variants_total": variants_total,
                "snps": snps,
                "indels": indels,
                "ti_tv": ti_tv,
                "call_summary": call_summary,
                "filter_summary": filter_summary,
                "filter_breakdown": filter_breakdown,
                "depth_distribution": depth_distribution,
            })),
        );
        let downstream_rows = ordered
            .iter()
            .filter(|row| {
                matches!(
                    row.stage_id.as_str(),
                    "vcf.pca"
                        | "vcf.population_structure"
                        | "vcf.roh"
                        | "vcf.ibd"
                        | "vcf.demography"
                        | "vcf.impute"
                )
            })
            .collect::<Vec<_>>();
        let downstream_sections = downstream_rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "stage_id": row.stage_id,
                    "tool_id": row.tool_id,
                    "runtime_s": row.runtime_s,
                    "metrics": {
                        "variant_density_per_mb": row.metrics.get("variant_density_per_mb").cloned().unwrap_or(serde_json::Value::Null),
                        "missingness_block_count": row.metrics.get("missingness_block_count").cloned().unwrap_or(serde_json::Value::Null),
                        "segment_count": row.metrics.get("segment_count").cloned().unwrap_or(serde_json::Value::Null),
                        "total_length": row.metrics.get("total_length").cloned().unwrap_or(serde_json::Value::Null),
                        "ibd_segment_count": row.metrics.get("ibd_segment_count").cloned().unwrap_or(serde_json::Value::Null),
                        "ibd_total_length_cM": row.metrics.get("ibd_total_length_cM").cloned().unwrap_or(serde_json::Value::Null),
                        "ne_recent": row.metrics.get("ne_recent").cloned().unwrap_or(serde_json::Value::Null),
                        "imputation_info_mean": row.metrics.get("imputation_info_mean").cloned().unwrap_or(serde_json::Value::Null),
                        "rsq_mean": row.metrics.get("rsq_mean").cloned().unwrap_or(serde_json::Value::Null),
                        "missingness_post": row.metrics.get("missingness_post").cloned().unwrap_or(serde_json::Value::Null),
                    }
                })
            })
            .collect::<Vec<_>>();
        let impute_qc_summary = downstream_rows
            .iter()
            .find(|row| row.stage_id == "vcf.impute")
            .map_or(serde_json::json!({}), |row| {
                serde_json::json!({
                    "imputation_info_mean": row.metrics.get("imputation_info_mean").cloned().unwrap_or(serde_json::Value::Null),
                    "rsq_mean": row.metrics.get("rsq_mean").cloned().unwrap_or(serde_json::Value::Null),
                    "missingness_pre": row.metrics.get("missingness_pre").cloned().unwrap_or(serde_json::Value::Null),
                    "missingness_post": row.metrics.get("missingness_post").cloned().unwrap_or(serde_json::Value::Null),
                    "shared_variants_count": row.metrics.get("shared_variants_count").cloned().unwrap_or(serde_json::Value::Null),
                })
            });
        sections.insert(
            "vcf_downstream".to_string(),
            JsonBlob::new(serde_json::json!({
                "schema_version": "bijux.vcf.downstream.reporting.v1",
                "qc_summary": impute_qc_summary,
                "stages": downstream_sections,
            })),
        );
    }
    if ordered.iter().any(|row| row.stage_id.starts_with(FASTQ_STAGE_PREFIX)) {
        sections.insert(
            "fastq".to_string(),
            JsonBlob::new(serde_json::json!({
                "schema_version": "bijux.report.section.fastq.v1",
                "stages": ordered.iter().filter(|row| row.stage_id.starts_with(FASTQ_STAGE_PREFIX)).count(),
            })),
        );
    }
    sections.insert("impact_metrics".to_string(), JsonBlob::new(impact_metrics_section(&ordered)));
    sections.insert("findings".to_string(), JsonBlob::new(findings_section(&ordered)));
    sections
        .insert("claims_registry".to_string(), JsonBlob::new(claims_registry_section(&ordered)));
    if let Some(handoff) = cross_domain_handoff_section(base_dir) {
        sections.insert("handoff".to_string(), JsonBlob::new(handoff));
    }
    sections.insert(
        "reproducibility".to_string(),
        JsonBlob::new(reproducibility_section(&ordered, &telemetry_events)),
    );
    sections
        .insert("pipeline_verdict".to_string(), JsonBlob::new(pipeline_verdict_section(&ordered)));
    sections.insert(
        "scientific_provenance".to_string(),
        JsonBlob::new(scientific_provenance_section(&ordered)),
    );
    sections.insert(
        "data_contract_validation".to_string(),
        JsonBlob::new(data_contract_validation_section(&completeness_clone)),
    );
    sections.insert("qc_delta".to_string(), JsonBlob::new(qc_delta_section(&ordered)));
    sections.insert("qc_artifacts".to_string(), JsonBlob::new(qc_artifacts_section(&ordered)));
    sections.insert(
        "contaminant_summary".to_string(),
        JsonBlob::new(contaminant_summary_section(&ordered)),
    );
    sections
        .insert("comparison_view".to_string(), JsonBlob::new(comparison_view_section(&ordered)));
    sections.insert("adapter_config".to_string(), JsonBlob::new(adapter_config_section(&ordered)));
    sections.insert("run_provenance".to_string(), JsonBlob::new(run_provenance_section(base_dir)));
    enforce_report_completeness_contract(&ordered, &sections)?;
    model.sections = sections;
    model.tables.insert("stage_completeness".to_string(), JsonBlob::new(stage_completeness));
    Ok(model)
}

/// Write a run-level report from facts rows.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_run_report_from_facts(base_dir: &Path, rows: &[FactsRowV1]) -> Result<PathBuf> {
    bijux_dna_infra::ensure_dir(base_dir)?;
    let path = base_dir.join("report.json");
    let model = build_run_report_model(base_dir, rows)?;
    write_report_json(&path, &model).context("write report.json")?;
    let html_path = base_dir.join("report.html");
    crate::report::render::html::write_report_html(&html_path, &model)
        .context("write report.html")?;
    let md_path = base_dir.join("report.md");
    crate::report::render::markdown::write_report_markdown(&md_path, &model)
        .context("write report.md")?;
    Ok(path)
}

/// Write a deterministic run summary JSON from facts rows.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_run_summary_from_facts(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    write_run_summary_json(path, rows)
}
