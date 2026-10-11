//! Semantic admission for aggregate FASTQ, BAM, and VCF benchmark reports.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::{
    BenchError, FullBenchmarkReport, FullBenchmarkReportRowStatus, FULL_BENCHMARK_REPORT_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullBenchmarkReportAdmissionContext {
    pub expected_result_ids: BTreeSet<String>,
    pub collector_missing_result_count: usize,
    pub collector_failure_count: usize,
    pub collector_unsupported_pair_count: usize,
    pub triggered_failure_class_count: usize,
}

/// Validate and admit an aggregate benchmark report against its source closure.
///
/// # Errors
/// Returns an error when source counts, row identity, section closure, or paths drift.
pub fn validate_full_benchmark_report(
    report: &mut FullBenchmarkReport,
    context: &FullBenchmarkReportAdmissionContext,
) -> Result<(), BenchError> {
    if report.schema_version != FULL_BENCHMARK_REPORT_SCHEMA_V1 {
        return invalid("unsupported full benchmark report schema");
    }
    require_path(&report.markdown_output_path, "markdown_output_path")?;
    require_path(&report.json_output_path, "json_output_path")?;
    if report.row_count != report.rows.len()
        || report.row_count
            != report.expected_result_row_count + report.explicit_unsupported_row_count
        || report.expected_result_row_count != context.expected_result_ids.len()
        || report.explicit_unsupported_row_count != context.collector_unsupported_pair_count
        || report.missing_result_row_count != context.collector_missing_result_count
        || report.failure_row_count != context.collector_failure_count
        || report.failure_class_row_count != context.triggered_failure_class_count
    {
        return invalid("full benchmark report source aggregates do not close");
    }

    let report_ids =
        report.rows.iter().map(|row| row.report_row_id.as_str()).collect::<BTreeSet<_>>();
    let result_ids =
        report.rows.iter().filter_map(|row| row.result_id.as_deref()).collect::<BTreeSet<_>>();
    if report_ids.len() != report.rows.len()
        || result_ids
            != context.expected_result_ids.iter().map(String::as_str).collect::<BTreeSet<_>>()
    {
        return invalid("full benchmark report row identities are incomplete or duplicated");
    }

    for row in &report.rows {
        for (value, field) in [
            (&row.report_row_id, "rows[].report_row_id"),
            (&row.domain, "rows[].domain"),
            (&row.stage_id, "rows[].stage_id"),
            (&row.tool_id, "rows[].tool_id"),
            (&row.corpus_id, "rows[].corpus_id"),
            (&row.asset_profile_id, "rows[].asset_profile_id"),
            (&row.report_section, "rows[].report_section"),
            (&row.evidence_path, "rows[].evidence_path"),
            (&row.detail, "rows[].detail"),
        ] {
            require_text(value, field)?;
        }
        require_path(&row.evidence_path, "rows[].evidence_path")?;
        if let Some(path) = &row.manifest_path {
            require_path(path, "rows[].manifest_path")?;
        }
        if row.stage_id.split_once('.').map(|(domain, _)| domain) != Some(row.domain.as_str())
            || matches!(row.row_status, FullBenchmarkReportRowStatus::UnsupportedPair)
                != row.result_id.is_none()
        {
            return invalid("full benchmark report row identity is inconsistent");
        }
    }

    let status_counts = status_counts(report);
    let domain_counts =
        report.rows.iter().fold(BTreeMap::<String, usize>::new(), |mut out, row| {
            *out.entry(row.domain.clone()).or_default() += 1;
            out
        });
    if report.row_status_counts != status_counts
        || report.domain_counts != domain_counts
        || report.present_row_count != status_count(report, FullBenchmarkReportRowStatus::Present)
        || report.missing_result_row_count
            != status_count(report, FullBenchmarkReportRowStatus::MissingResult)
        || report.unsupported_pair_row_count
            != status_count(report, FullBenchmarkReportRowStatus::UnsupportedPair)
        || report.unsupported_pair_row_count != report.explicit_unsupported_row_count
    {
        return invalid("full benchmark report status or domain aggregates do not close");
    }

    validate_sections(report, &report_ids)?;
    report.passes_behavior_test = true;
    Ok(())
}

fn validate_sections(
    report: &FullBenchmarkReport,
    report_ids: &BTreeSet<&str>,
) -> Result<(), BenchError> {
    if report.stage_centric_row_count != report.stage_centric.len()
        || report.tool_centric_row_count != report.tool_centric.len()
        || report.corpus_centric_row_count != report.corpus_centric.len()
        || report.pipeline_centric_row_count != report.pipeline_centric.rows.len()
        || report.pipeline_centric.row_count != report.pipeline_centric.rows.len()
        || report.runtime_row_count != report.runtime.len()
        || report.memory_row_count != report.memory.len()
        || report.runtime_row_count != report.row_count
        || report.memory_row_count != report.row_count
        || report.comparable_metric_row_count != report.comparable_metrics.len()
        || report.failure_row_count != report.failures.rows.len()
        || report.failure_class_row_count != report.failures.classification_rows.len()
        || report.missing_result_row_count != report.missing_results.len()
        || report.explicit_unsupported_row_count != report.unsupported_pairs.len()
    {
        return invalid(format!(
            "full benchmark report section aggregates do not close: stage={}/{}, tool={}/{}, corpus={}/{}, pipeline={}/{}, pipeline_rows={}/{}, runtime={}/{}, memory={}/{}, comparable={}/{}, failures={}/{}, classes={}/{}, missing={}/{}, unsupported={}/{}",
            report.stage_centric_row_count,
            report.stage_centric.len(),
            report.tool_centric_row_count,
            report.tool_centric.len(),
            report.corpus_centric_row_count,
            report.corpus_centric.len(),
            report.pipeline_centric_row_count,
            report.pipeline_centric.rows.len(),
            report.pipeline_centric.row_count,
            report.pipeline_centric.rows.len(),
            report.runtime_row_count,
            report.runtime.len(),
            report.memory_row_count,
            report.memory.len(),
            report.comparable_metric_row_count,
            report.comparable_metrics.len(),
            report.failure_row_count,
            report.failures.rows.len(),
            report.failure_class_row_count,
            report.failures.classification_rows.len(),
            report.missing_result_row_count,
            report.missing_results.len(),
            report.explicit_unsupported_row_count,
            report.unsupported_pairs.len(),
        ));
    }
    let runtime_ids =
        report.runtime.iter().map(|row| row.report_row_id.as_str()).collect::<BTreeSet<_>>();
    let memory_ids =
        report.memory.iter().map(|row| row.report_row_id.as_str()).collect::<BTreeSet<_>>();
    if &runtime_ids != report_ids || &memory_ids != report_ids {
        return invalid("full benchmark report runtime or memory identity closure drifted");
    }
    if report.missing_results.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>()
        != report
            .rows
            .iter()
            .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
            .filter_map(|row| row.result_id.as_deref())
            .collect::<BTreeSet<_>>()
    {
        return invalid("full benchmark report missing-result section drifted");
    }
    let unsupported = report
        .unsupported_pairs
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    let unsupported_rows = report
        .rows
        .iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
        .map(|row| (row.domain.as_str(), row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    if unsupported != unsupported_rows {
        return invalid("full benchmark report unsupported-pair section drifted");
    }
    let failure_domains = report.failures.rows.iter().fold(BTreeMap::new(), |mut out, row| {
        *out.entry(row.domain.clone()).or_default() += 1;
        out
    });
    if report.failures.simulated_failure_row_count != report.failures.rows.len()
        || report.failures.failure_class_row_count != report.failures.classification_rows.len()
        || report.failures.simulated_failure_domain_counts != failure_domains
        || report.failures.classification_rows.iter().any(|row| !row.triggered)
    {
        return invalid("full benchmark report failure section drifted");
    }
    if report.missing_result_row_count == 0 || report.unsupported_pair_row_count == 0 {
        return invalid("full benchmark report must preserve missing and unsupported rows");
    }
    Ok(())
}

fn status_counts(report: &FullBenchmarkReport) -> BTreeMap<String, usize> {
    report.rows.iter().fold(BTreeMap::new(), |mut out, row| {
        *out.entry(status_label(row.row_status).to_string()).or_default() += 1;
        out
    })
}

fn status_count(report: &FullBenchmarkReport, status: FullBenchmarkReportRowStatus) -> usize {
    report.rows.iter().filter(|row| row.row_status == status).count()
}

fn status_label(status: FullBenchmarkReportRowStatus) -> &'static str {
    match status {
        FullBenchmarkReportRowStatus::Present => "present",
        FullBenchmarkReportRowStatus::MissingResult => "missing_result",
        FullBenchmarkReportRowStatus::UnsupportedPair => "unsupported_pair",
    }
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("full benchmark report requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!("full benchmark report requires safe repository-relative {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
