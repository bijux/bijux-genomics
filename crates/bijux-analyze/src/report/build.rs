use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::aggregate::{
    derived_metric_spec, derived_metrics_for_stage, metric_kind_for_stage, metric_spec,
    stage_metric_spec, BenchmarkRecord, DerivedMetricId, FastqCorrectMetrics, FastqFilterMetrics,
    FastqMergeMetrics, FastqQcPostMetrics, FastqStatsMetrics, FastqTrimMetrics, FastqUmiMetrics,
    FastqValidateMetrics,
};
use crate::decision::score::{build_rankings, RankInput};
use crate::facts::write_run_summary_json;
use crate::failure::{classify_raw_failure, BenchmarkFailure};
use crate::model::JsonBlob;
use crate::semantic::{semantic_filter, semantic_stats, semantic_trim, semantic_validate};
use bijux_core::observability::QcPostReportV1;
use bijux_core::{
    AssetsProvenanceV1, FactsRowV1, FilterReportV1, MetricSemanticsV1, RawFailure,
    ReportCompletenessV1, ReportContractV1, ReportProvenanceV1, ReportSchemaV1,
    ReportStageSummaryV1, RetentionContextV1, RetentionDefinitionV1, RetentionReportV1,
    StageReportV1, TelemetryEventV1,
};

use super::model::ReportModel;
use super::render_json::write_report_json;

/// Write the trim benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_trim_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqTrimMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_trim(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_trim_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_trim(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let rankings = rank_trim_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.trim", &rankings);
    }
    Ok(())
}

/// Write the validate benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_validate_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqValidateMetrics>],
    failures: &[RawFailure],
    qc_class: Option<&str>,
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_validate(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_validate(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    if let Some(class) = qc_class {
        report.insert("qc_class", serde_json::to_value(class)?);
    }
    let rankings = rank_validate_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.validate_pre", &rankings);
    }
    Ok(())
}

/// Write the filter benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_filter_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqFilterMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_filter(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_filter(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let derived: Vec<_> = records.iter().map(derived_filter_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_filter_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.filter", &rankings);
    }
    Ok(())
}

/// Write the merge benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_merge_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqMergeMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_merge(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_merge_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_merge_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.merge", &rankings);
    }
    Ok(())
}

/// Write the correct benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_correct_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_correct(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_correct_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_correct_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.correct", &rankings);
    }
    Ok(())
}

/// Write the `qc_post` benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_qc_post_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqQcPostMetrics>],
    failures: &[RawFailure],
    qc_class: Option<&str>,
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_qc_post(records))?,
    );
    if let Some(class) = qc_class {
        report.insert("qc_class", serde_json::to_value(class)?);
    }
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.qc_post", &BTreeMap::new());
    }
    Ok(())
}

/// Write the umi benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_umi_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqUmiMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_umi(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_umi_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_umi_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.umi", &rankings);
    }
    Ok(())
}

/// Write the stats benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_stats_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqStatsMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_stats(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_stats(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.stats_neutral", &BTreeMap::new());
    }
    Ok(())
}

/// Write a run-level report from facts rows.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
#[allow(clippy::too_many_lines)]
pub fn write_run_report_from_facts(base_dir: &Path, rows: &[FactsRowV1]) -> Result<PathBuf> {
    let mut ordered = rows.to_vec();
    crate::facts::stable_sort_records(&mut ordered, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
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

    let path = base_dir.join("report.json");
    let sections = build_report_sections(&report)
        .into_iter()
        .map(|(key, value)| (key, JsonBlob::new(value)))
        .collect();
    let mut model = ReportModel::empty(report);
    model.sections = sections;
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

fn report_path_for(reports: &serde_json::Value, key: &str) -> Option<String> {
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
        let Ok(raw) = fs::read_to_string(path) else {
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

fn qc_improvement_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            report_path = report_path_for(&row.reports, "qc_post_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<QcPostReportV1>(&raw) else {
        return serde_json::json!({});
    };
    let module_names = [
        "Per base sequence quality",
        "Per sequence quality scores",
        "Per sequence GC content",
        "Adapter Content",
        "Sequence Duplication Levels",
        "Overrepresented sequences",
    ];
    let mut entries = serde_json::Map::new();
    for module in module_names {
        let raw_status = report
            .fastqc_raw_modules
            .get(module)
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let trimmed_status = report
            .fastqc_trimmed_modules
            .get(module)
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let delta = match (raw_status.as_deref(), trimmed_status.as_deref()) {
            (Some(raw), Some(trimmed)) => {
                let score = |status: &str| match status {
                    "PASS" => 2,
                    "WARN" => 1,
                    _ => 0,
                };
                let before = score(raw);
                let after = score(trimmed);
                match after.cmp(&before) {
                    Ordering::Greater => "improved",
                    Ordering::Less => "regressed",
                    Ordering::Equal => "unchanged",
                }
            }
            _ => "unknown",
        };
        entries.insert(
            module.to_string(),
            serde_json::json!({
                "raw_status": raw_status,
                "trimmed_status": trimmed_status,
                "delta": delta,
            }),
        );
    }
    serde_json::json!({
        "raw_fastqc_dir": report.raw_fastqc_dir,
        "trimmed_fastqc_dir": report.trimmed_fastqc_dir,
        "multiqc_report": report.multiqc_report,
        "multiqc_data": report.multiqc_data,
        "modules": entries,
    })
}

fn filter_interpretation_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.filter" {
            report_path = report_path_for(&row.reports, "filter_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<FilterReportV1>(&raw) else {
        return serde_json::json!({});
    };
    serde_json::json!({
        "why_this_matters": "Filtering removes reads that are likely to be low-quality, low-complexity, or contaminant-derived, improving downstream accuracy.",
        "recommended_ranges": {
            "ancient_dna": {
                "max_n": "0-2",
                "low_complexity_threshold": "0.4-0.6",
                "kmer_ref": "enable if contaminant references are available",
            },
            "modern_ngs": {
                "max_n": "0",
                "low_complexity_threshold": "0.6-0.8",
                "kmer_ref": "enable for known contaminant panels (e.g. PhiX/UniVec)",
            }
        },
        "conditions": report.conditions,
        "removed_breakdown": {
            "total": report.reads_removed_total,
            "by_n": report.reads_removed_by_n,
            "by_entropy": report.reads_removed_by_entropy,
            "by_low_complexity": report.reads_removed_low_complexity,
            "by_kmer": report.reads_removed_by_kmer,
            "by_contaminant_kmer": report.reads_removed_contaminant_kmer,
            "by_length": report.reads_removed_by_length,
        },
        "entropy_distribution": report.entropy_distribution,
        "redundant_filters": report.redundant_filters,
    })
}

fn adapter_inference_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            report_path = report_path_for(&row.reports, "qc_post_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<QcPostReportV1>(&raw) else {
        return serde_json::json!({});
    };
    let suggestions = report
        .suggested_adapters_path
        .as_deref()
        .and_then(|path| {
            let raw = fs::read_to_string(path).ok()?;
            serde_json::from_str::<serde_json::Value>(&raw).ok()
        })
        .unwrap_or_else(|| serde_json::json!({}));
    let rationale = match report.suggested_preset.as_deref() {
        Some("illumina_twocolor") => {
            "PolyG/overrepresented sequences consistent with two-color chemistry."
        }
        Some("ssdna") => "Overrepresented adapter motifs match ssDNA library prep.",
        Some(_) => "Adapter motifs detected in overrepresented sequences.",
        None => "No strong adapter signal detected.",
    };
    serde_json::json!({
        "suggested_preset": report.suggested_preset,
        "suggested_adapters": suggestions,
        "rationale": rationale,
        "safety": "Inference never changes trimming unless --accept-suggested-adapters is set.",
    })
}

fn report_contract() -> ReportContractV1 {
    ReportContractV1 {
        schema_version: "bijux.report_contract.v1".to_string(),
        required_sections: vec![
            "contract".to_string(),
            "completeness".to_string(),
            "stages".to_string(),
            "provenance".to_string(),
            "retention_definition".to_string(),
            "retention_context".to_string(),
            "assets_provenance".to_string(),
            "metric_semantics".to_string(),
            "telemetry".to_string(),
            "qc_improvement".to_string(),
            "final_qc_summary".to_string(),
            "filter_interpretation".to_string(),
            "adapter_inference".to_string(),
        ],
        required_provenance_fields: vec![
            "tool_id".to_string(),
            "tool_version".to_string(),
            "image_digest".to_string(),
            "trace_id".to_string(),
            "span_id".to_string(),
            "params_hash".to_string(),
            "bank_hashes".to_string(),
        ],
    }
}

fn build_report_sections(report: &ReportSchemaV1) -> BTreeMap<String, serde_json::Value> {
    let mut sections = BTreeMap::new();
    sections.insert("qc".to_string(), report.qc_improvement.clone());
    sections.insert(
        "final_qc_summary".to_string(),
        report.final_qc_summary.clone(),
    );
    sections.insert(
        "trimming".to_string(),
        serde_json::json!({
            "retention_definition": report.retention_definition.clone(),
            "retention_context": report.retention_context.clone(),
        }),
    );
    sections.insert(
        "filtering".to_string(),
        report.filter_interpretation.clone(),
    );
    sections.insert(
        "contamination".to_string(),
        serde_json::json!({
            "assets": report.assets_provenance.clone(),
        }),
    );
    sections.insert(
        "retention".to_string(),
        serde_json::json!({
            "definitions": report.retention_definition.clone(),
            "contexts": report.retention_context.clone(),
        }),
    );
    sections.insert(
        "failures".to_string(),
        serde_json::json!({
            "completeness": report.completeness,
        }),
    );
    sections
}

fn report_completeness(
    missing_metrics: &[String],
    missing_reports: &[String],
) -> ReportCompletenessV1 {
    let status = if missing_metrics.is_empty() && missing_reports.is_empty() {
        "complete"
    } else {
        "incomplete"
    };
    ReportCompletenessV1 {
        status: status.to_string(),
        missing_metrics: missing_metrics.to_vec(),
        missing_reports: missing_reports.to_vec(),
    }
}

fn report_metric_semantics() -> Vec<MetricSemanticsV1> {
    let metric_ids = [
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "merge_rate",
        "error_reduction_proxy",
    ];
    metric_ids
        .iter()
        .filter_map(|metric_id| {
            bijux_core::metric_semantics(metric_id).map(|spec| MetricSemanticsV1 {
                metric_id: spec.metric_id.to_string(),
                direction: format!("{:?}", spec.direction),
                units: spec.units.to_string(),
                range: spec.range.to_string(),
                missing_data_policy: spec.missing_data_policy.to_string(),
            })
        })
        .collect()
}

fn gate_payload(failures: &[BenchmarkFailure]) -> serde_json::Value {
    let rationale: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "kind": format!("{:?}", failure.kind),
            })
        })
        .collect();
    serde_json::json!({
        "passes": failures.is_empty(),
        "rationale": rationale
    })
}

/// Print the benchmark schema for a stage.
///
/// # Errors
/// Returns an error if the schema cannot be rendered.
pub fn print_bench_schema(stage: &str) -> Result<()> {
    let json = bench_schema_json(stage)?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn bench_schema_json(stage: &str) -> Result<serde_json::Value> {
    let kind = metric_kind_for_stage(stage).ok_or_else(|| anyhow!("unknown stage {stage}"))?;
    let spec = stage_metric_spec(kind);
    let metrics: Vec<_> = spec
        .metrics
        .iter()
        .map(|metric_id| {
            let metric = metric_spec(*metric_id);
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
                "direction": format!("{:?}", metric.direction),
                "range": metric.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max
                })),
                "measured": metric.measured,
                "derived": metric.derived,
            })
        })
        .collect();
    let derived: Vec<_> = derived_metrics_for_stage(stage)
        .into_iter()
        .map(|metric| {
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
                "direction": format!("{:?}", metric.direction),
                "range": metric.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max
                })),
                "derived": true,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "stage": spec.stage,
        "metrics": metrics,
        "derived_metrics": derived,
        "invariants": spec.invariants,
    }))
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        f64::midpoint(values[mid - 1], values[mid])
    } else {
        values[mid]
    }
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        u64_to_f64(num) / u64_to_f64(denom)
    }
}

fn sanity_flags_trim(records: &[BenchmarkRecord<FastqTrimMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.metrics.reads_in > 0 {
                let retention = ratio_u64(
                    record.metrics.metrics.reads_out,
                    record.metrics.metrics.reads_in,
                );
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_filter(records: &[BenchmarkRecord<FastqFilterMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.metrics.reads_in > 0 {
                let retention = ratio_u64(
                    record.metrics.metrics.reads_out,
                    record.metrics.metrics.reads_in,
                );
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_correct(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.metrics.reads_in > 0 {
                let retention = ratio_u64(
                    record.metrics.metrics.reads_out,
                    record.metrics.metrics.reads_in,
                );
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_umi(records: &[BenchmarkRecord<FastqUmiMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.metrics.reads_in > 0 {
                let retention = ratio_u64(
                    record.metrics.metrics.reads_out,
                    record.metrics.metrics.reads_in,
                );
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_merge(records: &[BenchmarkRecord<FastqMergeMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_stats(records: &[BenchmarkRecord<FastqStatsMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    let gc_median = median(
        records
            .iter()
            .map(|r| r.metrics.metrics.gc_percent)
            .collect(),
    );
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if (record.metrics.metrics.gc_percent - gc_median).abs() > 10.0 {
                flags.push("gc_shift_gt_10");
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_validate(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_qc_post(records: &[BenchmarkRecord<FastqQcPostMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn derived_trim_metrics(record: &BenchmarkRecord<FastqTrimMetrics>) -> serde_json::Value {
    let delta = &record.metrics.metrics.delta_metrics;
    let error_reduction_proxy = delta.mean_q_delta.max(0.0);
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: delta.read_retention,
        derived_metric_spec(DerivedMetricId::BaseRetention).name: delta.base_retention,
        derived_metric_spec(DerivedMetricId::ErrorReductionProxy).name: error_reduction_proxy,
    })
}

fn derived_filter_metrics(record: &BenchmarkRecord<FastqFilterMetrics>) -> serde_json::Value {
    let delta = &record.metrics.metrics.delta_metrics;
    let error_reduction_proxy = delta.mean_q_delta.max(0.0);
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: delta.read_retention,
        derived_metric_spec(DerivedMetricId::BaseRetention).name: delta.base_retention,
        derived_metric_spec(DerivedMetricId::ErrorReductionProxy).name: error_reduction_proxy,
    })
}

fn derived_merge_metrics(record: &BenchmarkRecord<FastqMergeMetrics>) -> serde_json::Value {
    let metrics = &record.metrics.metrics;
    let merged_ratio = metrics.merge_rate;
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::MergeEfficiency).name: merged_ratio,
    })
}

fn derived_correct_metrics(record: &BenchmarkRecord<FastqCorrectMetrics>) -> serde_json::Value {
    let metrics = &record.metrics.metrics;
    let read_retention = if metrics.reads_in == 0 {
        0.0
    } else {
        u64_to_f64(metrics.reads_out) / u64_to_f64(metrics.reads_in)
    };
    let base_retention = if metrics.bases_in == 0 {
        0.0
    } else {
        u64_to_f64(metrics.bases_out) / u64_to_f64(metrics.bases_in)
    };
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: read_retention,
        derived_metric_spec(DerivedMetricId::BaseRetention).name: base_retention,
    })
}

fn derived_umi_metrics(record: &BenchmarkRecord<FastqUmiMetrics>) -> serde_json::Value {
    let metrics = &record.metrics.metrics;
    let retention = if metrics.reads_in == 0 {
        0.0
    } else {
        u64_to_f64(metrics.reads_out) / u64_to_f64(metrics.reads_in)
    };
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: retention,
    })
}

fn rank_trim_tools(
    records: &[BenchmarkRecord<FastqTrimMetrics>],
) -> Result<BTreeMap<String, Vec<crate::decision::score::RankingEntry>>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(record.metrics.metrics.delta_metrics.read_retention),
            base_retention: Some(record.metrics.metrics.delta_metrics.base_retention),
            error_reduction_proxy: Some(record.metrics.metrics.delta_metrics.mean_q_delta.max(0.0)),
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_validate_tools(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Result<BTreeMap<String, Vec<crate::decision::score::RankingEntry>>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_filter_tools(
    records: &[BenchmarkRecord<FastqFilterMetrics>],
) -> Result<BTreeMap<String, Vec<crate::decision::score::RankingEntry>>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(record.metrics.metrics.delta_metrics.read_retention),
            base_retention: Some(record.metrics.metrics.delta_metrics.base_retention),
            error_reduction_proxy: Some(record.metrics.metrics.delta_metrics.mean_q_delta.max(0.0)),
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_merge_tools(
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> Result<BTreeMap<String, Vec<crate::decision::score::RankingEntry>>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(record.metrics.metrics.merge_rate),
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_correct_tools(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Result<BTreeMap<String, Vec<crate::decision::score::RankingEntry>>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(if record.metrics.metrics.reads_in == 0 {
                0.0
            } else {
                u64_to_f64(record.metrics.metrics.reads_out)
                    / u64_to_f64(record.metrics.metrics.reads_in)
            }),
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_umi_tools(
    records: &[BenchmarkRecord<FastqUmiMetrics>],
) -> Result<BTreeMap<String, Vec<crate::decision::score::RankingEntry>>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(if record.metrics.metrics.reads_in == 0 {
                0.0
            } else {
                u64_to_f64(record.metrics.metrics.reads_out)
                    / u64_to_f64(record.metrics.metrics.reads_in)
            }),
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::BenchmarkContext;
    use bijux_core::measure::ExecutionMetrics;
    use bijux_core::metrics::MetricSet;

    #[test]
    fn bench_schema_table_has_metrics() -> Result<()> {
        let json = bench_schema_json("fastq.trim")?;
        assert_eq!(json["stage"], "fastq.trim");
        assert!(!json["metrics"].as_array().unwrap_or(&Vec::new()).is_empty());
        Ok(())
    }

    #[test]
    fn bench_schema_table_ordering_matches_registry() -> Result<()> {
        let json = bench_schema_json("fastq.trim")?;
        let observed: Vec<_> = json["metrics"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|entry| entry["name"].as_str())
            .map(ToString::to_string)
            .collect();
        let kind = metric_kind_for_stage("fastq.trim").ok_or_else(|| anyhow!("stage kind"))?;
        let spec = stage_metric_spec(kind);
        let expected: Vec<_> = spec
            .metrics
            .iter()
            .map(|metric_id| metric_spec(*metric_id).name.to_string())
            .collect();
        assert_eq!(observed, expected);
        Ok(())
    }

    #[test]
    fn bench_schema_table_omits_range_when_missing() -> Result<()> {
        let json = bench_schema_json("fastq.trim")?;
        let empty = Vec::new();
        let entry = json["metrics"]
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .find(|metric| metric["name"] == "delta_metrics")
            .ok_or_else(|| anyhow!("delta_metrics"))?;
        assert!(entry.get("range").is_some());
        assert!(entry["range"].is_null());
        Ok(())
    }

    #[test]
    fn run_summary_aggregation_works() -> Result<()> {
        let dir = tempfile::TempDir::new()?;
        let rows = vec![FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-1".to_string(),
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: Some("sha256:abc".to_string()),
            trace_id: "trace-1".to_string(),
            span_id: "span-1".to_string(),
            params_hash: "ph".to_string(),
            input_hash: "ih".to_string(),
            output_hashes: vec!["oh".to_string()],
            runtime_s: 1.0,
            memory_mb: 32.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({}),
            reads_in: Some(10),
            reads_out: Some(9),
            bases_in: Some(100),
            bases_out: Some(90),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({}),
            artifacts: serde_json::json!({}),
        }];
        let summary_path = dir.path().join("run_summary.json");
        write_run_summary_from_facts(&summary_path, &rows)?;
        let summary_raw = std::fs::read_to_string(summary_path)?;
        let summary_value: serde_json::Value = serde_json::from_str(&summary_raw)?;
        assert_eq!(summary_value["runs"], 1);
        assert_eq!(summary_value["stages"], 1);
        Ok(())
    }

    #[test]
    fn ranking_explanation_generation_has_modes() -> Result<()> {
        let metrics = FastqTrimMetrics {
            reads_in: 100,
            reads_out: 90,
            bases_in: 1000,
            bases_out: 900,
            pairs_in: None,
            pairs_out: None,
            mean_q_before: 30.0,
            mean_q_after: 31.0,
            delta_metrics: crate::FastqDeltaMetrics {
                read_retention: 0.9,
                base_retention: 0.9,
                mean_q_delta: 1.0,
                gc_delta: 0.1,
            },
            adapter_preset: None,
            adapter_bank_id: None,
            adapter_bank_hash: None,
            adapter_overrides: None,
        };
        let record = BenchmarkRecord {
            context: BenchmarkContext {
                tool: "fastp".to_string(),
                tool_version: "0.23.4".to_string(),
                image_digest: "sha256:abc".to_string(),
                runner: "docker".to_string(),
                platform: "linux".to_string(),
                input_hash: "ih".to_string(),
                parameters: crate::model::JsonBlob::default(),
            },
            execution: ExecutionMetrics {
                runtime_s: 1.0,
                memory_mb: 10.0,
                exit_code: 0,
            },
            metrics: MetricSet {
                metrics_schema: "fastq_trim_v2".to_string(),
                version: 2,
                metrics,
            },
        };
        let rankings = rank_trim_tools(&[record])?;
        assert!(rankings.contains_key("FastestAcceptable"));
        assert!(rankings.contains_key("MostConservative"));
        assert!(rankings.contains_key("BalancedPareto"));
        Ok(())
    }
}
