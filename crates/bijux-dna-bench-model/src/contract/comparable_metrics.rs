//! Owner: bijux-dna-bench-model
//! Semantic validation for persisted comparable-metric readiness reports.

use std::collections::BTreeSet;

use crate::{
    model::{
        BamComparableMetricsReport, ComparableMetricContractStatus, FastqComparableMetricsReport,
        VcfComparableMetricsReport,
    },
    BenchError,
};

use super::{
    BAM_COMPARABLE_METRICS_SCHEMA_V1, FASTQ_COMPARABLE_METRICS_SCHEMA_V1,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};

/// Validate one FASTQ comparable-metric readiness report.
///
/// # Errors
/// Returns an error when identity, aggregate counts, stage rows, or metric closure drift.
pub fn validate_fastq_comparable_metrics_report(
    report: &FastqComparableMetricsReport,
) -> Result<(), BenchError> {
    if report.schema_version != FASTQ_COMPARABLE_METRICS_SCHEMA_V1 {
        return invalid("unsupported FASTQ comparable-metrics schema");
    }
    validate_common_report(
        &report.output_path,
        ComparableMetricReportCounters::from(report),
        report.rows.iter().map(ComparableMetricRowView::from),
    )
}

/// Validate one BAM comparable-metric readiness report.
///
/// # Errors
/// Returns an error when identity, aggregate counts, stage rows, or metric closure drift.
pub fn validate_bam_comparable_metrics_report(
    report: &BamComparableMetricsReport,
) -> Result<(), BenchError> {
    if report.schema_version != BAM_COMPARABLE_METRICS_SCHEMA_V1 {
        return invalid("unsupported BAM comparable-metrics schema");
    }
    validate_common_report(
        &report.output_path,
        ComparableMetricReportCounters::from(report),
        report.rows.iter().map(ComparableMetricRowView::from),
    )
}

#[derive(Debug, Clone, Copy)]
struct ComparableMetricReportCounters {
    comparable_stages: usize,
    multi_tool_stages: usize,
    comparable_tool_rows: usize,
    rows: usize,
    declared_stages: usize,
    missing_shared_metric_stages: usize,
    shared_metric_fields: usize,
}

impl From<&FastqComparableMetricsReport> for ComparableMetricReportCounters {
    fn from(report: &FastqComparableMetricsReport) -> Self {
        Self {
            comparable_stages: report.comparable_stage_count,
            multi_tool_stages: report.multi_tool_stage_count,
            comparable_tool_rows: report.comparable_tool_row_count,
            rows: report.row_count,
            declared_stages: report.declared_stage_count,
            missing_shared_metric_stages: report.missing_shared_metric_stage_count,
            shared_metric_fields: report.shared_metric_field_count,
        }
    }
}

impl From<&BamComparableMetricsReport> for ComparableMetricReportCounters {
    fn from(report: &BamComparableMetricsReport) -> Self {
        Self {
            comparable_stages: report.comparable_stage_count,
            multi_tool_stages: report.multi_tool_stage_count,
            comparable_tool_rows: report.comparable_tool_row_count,
            rows: report.row_count,
            declared_stages: report.declared_stage_count,
            missing_shared_metric_stages: report.missing_shared_metric_stage_count,
            shared_metric_fields: report.shared_metric_field_count,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ComparableMetricRowView<'a> {
    stage: &'a str,
    status: ComparableMetricContractStatus,
    tool_count: usize,
    tools: &'a [String],
    default_tool: &'a str,
    corpus_status: &'a str,
    field_count: usize,
    fields: &'a [String],
    reason: &'a str,
}

impl<'a> From<&'a crate::model::FastqComparableMetricsRow> for ComparableMetricRowView<'a> {
    fn from(row: &'a crate::model::FastqComparableMetricsRow) -> Self {
        Self {
            stage: &row.stage_id,
            status: row.comparison_contract_status,
            tool_count: row.tool_count,
            tools: &row.tool_ids,
            default_tool: &row.default_tool_id,
            corpus_status: &row.corpus_status,
            field_count: row.shared_metric_field_count,
            fields: &row.shared_metric_fields,
            reason: &row.reason,
        }
    }
}

impl<'a> From<&'a crate::model::BamComparableMetricsRow> for ComparableMetricRowView<'a> {
    fn from(row: &'a crate::model::BamComparableMetricsRow) -> Self {
        Self {
            stage: &row.stage_id,
            status: row.comparison_contract_status,
            tool_count: row.tool_count,
            tools: &row.tool_ids,
            default_tool: &row.default_tool_id,
            corpus_status: &row.corpus_status,
            field_count: row.shared_metric_field_count,
            fields: &row.shared_metric_fields,
            reason: &row.reason,
        }
    }
}

fn validate_common_report<'a>(
    output_path: &str,
    counters: ComparableMetricReportCounters,
    rows: impl Iterator<Item = ComparableMetricRowView<'a>>,
) -> Result<(), BenchError> {
    require_text(output_path, "output_path")?;
    let rows = rows.collect::<Vec<_>>();
    if counters.rows != rows.len() || counters.multi_tool_stages != rows.len() {
        return invalid("comparable-metrics row aggregates do not close");
    }
    if counters.declared_stages + counters.missing_shared_metric_stages != rows.len() {
        return invalid("comparable-metrics status aggregates do not close");
    }
    if counters.comparable_stages < counters.multi_tool_stages
        || counters.comparable_tool_rows < counters.multi_tool_stages.saturating_mul(2)
    {
        return invalid("comparable-metrics stage or tool aggregates are inconsistent");
    }
    let mut stages = BTreeSet::new();
    let mut observed_declared = 0;
    let mut observed_missing = 0;
    let mut observed_fields = 0;
    let mut observed_multi_tool_rows = 0;
    for row in rows {
        for (value, field) in [
            (row.stage, "rows[].stage_id"),
            (row.default_tool, "rows[].default_tool_id"),
            (row.corpus_status, "rows[].corpus_status"),
            (row.reason, "rows[].reason"),
        ] {
            require_text(value, field)?;
        }
        if !stages.insert(row.stage) || row.tool_count != row.tools.len() || row.tool_count < 2 {
            return invalid("comparable-metrics tool coverage is invalid");
        }
        validate_unique_text(row.tools, "rows[].tool_ids")?;
        if !row.tools.iter().any(|tool| tool == row.default_tool) {
            return invalid("comparable-metrics default tool is outside the tool roster");
        }
        if row.field_count != row.fields.len() {
            return invalid("comparable-metrics shared metric count does not close");
        }
        validate_unique_text_or_empty(row.fields, "rows[].shared_metric_fields")?;
        match row.status {
            ComparableMetricContractStatus::Declared if row.fields.is_empty() => {
                return invalid("declared comparable-metrics row has no shared metrics");
            }
            ComparableMetricContractStatus::Declared => observed_declared += 1,
            ComparableMetricContractStatus::MissingSharedMetrics if !row.fields.is_empty() => {
                return invalid("missing comparable-metrics row declares shared metrics");
            }
            ComparableMetricContractStatus::MissingSharedMetrics => observed_missing += 1,
        }
        observed_multi_tool_rows += row.tool_count;
        observed_fields += row.fields.len();
    }
    let observed_single_tool_rows = counters.comparable_stages - counters.multi_tool_stages;
    if observed_declared != counters.declared_stages
        || observed_missing != counters.missing_shared_metric_stages
        || observed_fields != counters.shared_metric_fields
        || observed_multi_tool_rows + observed_single_tool_rows != counters.comparable_tool_rows
    {
        return invalid("comparable-metrics observed aggregates do not match the report");
    }
    Ok(())
}

/// Validate one VCF comparable-metric readiness report.
///
/// # Errors
/// Returns an error when stage coverage, metric identities, or aggregate counts drift.
pub fn validate_vcf_comparable_metrics_report(
    report: &VcfComparableMetricsReport,
) -> Result<(), BenchError> {
    if report.schema_version != VCF_COMPARABLE_METRICS_SCHEMA_V1 {
        return invalid("unsupported VCF comparable-metrics schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.row_count != report.rows.len() || report.rows.is_empty() {
        return invalid("VCF comparable-metrics row aggregate does not close");
    }
    let mut stages = BTreeSet::new();
    let mut metrics = BTreeSet::new();
    let mut stage_tools = std::collections::BTreeMap::<&str, &[String]>::new();
    for row in &report.rows {
        for (value, field) in [
            (row.stage_id.as_str(), "rows[].stage_id"),
            (row.metric_id.as_str(), "rows[].metric_id"),
            (row.metric_name.as_str(), "rows[].metric_name"),
            (row.unit.as_str(), "rows[].unit"),
        ] {
            require_text(value, field)?;
        }
        if !metrics.insert((row.stage_id.as_str(), row.metric_id.as_str())) {
            return invalid("VCF comparable-metrics repeats a stage/metric identity");
        }
        validate_unique_text(&row.tools_covered, "rows[].tools_covered")?;
        if row.tools_covered.len() < 2 {
            return invalid("VCF comparable-metrics row covers fewer than two tools");
        }
        if let Some(expected) = stage_tools.insert(&row.stage_id, &row.tools_covered) {
            if expected != row.tools_covered {
                return invalid("VCF comparable-metrics tool roster drifts within a stage");
            }
        }
        stages.insert(row.stage_id.as_str());
    }
    let retained_tool_rows = stage_tools.values().map(|tools| tools.len()).sum::<usize>();
    if report.stage_count != stages.len()
        || report.multi_tool_stage_count != stages.len()
        || report.retained_tool_row_count != retained_tool_rows
    {
        return invalid("VCF comparable-metrics stage or tool aggregates do not close");
    }
    Ok(())
}

fn validate_unique_text(values: &[String], field: &str) -> Result<(), BenchError> {
    if values.is_empty()
        || values.iter().any(|value| value.trim().is_empty())
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        return invalid(format!("comparable-metrics {field} is invalid"));
    }
    Ok(())
}

fn validate_unique_text_or_empty(values: &[String], field: &str) -> Result<(), BenchError> {
    if values.iter().any(|value| value.trim().is_empty())
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        return invalid(format!("comparable-metrics {field} is invalid"));
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("comparable-metrics requires non-empty {field}"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
