//! Admission invariants for the all-domain expected benchmark result inventory.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AllDomainExpectedBenchmarkResultRow, AllDomainExpectedBenchmarkResultsReport, BenchError,
    ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1,
};

/// Validates the complete expected benchmark result inventory and its aggregate counts.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when schema identity, row identity, paths, row
/// semantics, or aggregate counts are inconsistent.
pub fn validate_all_domain_expected_benchmark_results(
    report: &AllDomainExpectedBenchmarkResultsReport,
) -> Result<(), BenchError> {
    if report.schema_version != ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1 {
        return invalid("all-domain expected benchmark result schema identity is invalid");
    }
    if !safe_relative_path(&report.output_path) || report.rows.is_empty() {
        return invalid("all-domain expected benchmark result output or rows are invalid");
    }
    let mut result_ids = BTreeSet::new();
    for row in &report.rows {
        validate_row(row)?;
        if !result_ids.insert(row.result_id.as_str()) {
            return invalid("all-domain expected benchmark result identity is duplicated");
        }
    }
    if report.rows.windows(2).any(|rows| row_key(&rows[0]) >= row_key(&rows[1])) {
        return invalid("all-domain expected benchmark result rows are not canonically ordered");
    }
    if report.row_count != report.rows.len()
        || report.result_id_count != result_ids.len()
        || report.stage_count != unique_count(&report.rows, |row| row.stage_id.as_str())
        || report.tool_count != unique_count(&report.rows, |row| row.tool_id.as_str())
        || report.corpus_count != unique_count(&report.rows, |row| row.corpus_id.as_str())
        || report.asset_profile_count
            != unique_count(&report.rows, |row| row.asset_profile_id.as_str())
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.report_section_counts != count_by(&report.rows, |row| row.report_section.as_str())
    {
        return invalid("all-domain expected benchmark result aggregates are inconsistent");
    }
    Ok(())
}

fn validate_row(row: &AllDomainExpectedBenchmarkResultRow) -> Result<(), BenchError> {
    if [
        row.result_id.as_str(),
        row.domain.as_str(),
        row.stage_id.as_str(),
        row.tool_id.as_str(),
        row.corpus_id.as_str(),
        row.asset_profile_id.as_str(),
        row.report_section.as_str(),
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || !matches!(row.domain.as_str(), "bam" | "fastq" | "vcf")
        || row.result_id
            != format!(
                "{}:{}:{}:{}:{}",
                row.domain, row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id
            )
        || row.expected_outputs.is_empty()
        || row.expected_metrics.is_empty()
        || has_blank_or_duplicate(&row.expected_outputs)
        || has_blank_or_duplicate(&row.expected_metrics)
    {
        return invalid("all-domain expected benchmark result row has invalid semantics");
    }
    Ok(())
}

fn row_key(row: &AllDomainExpectedBenchmarkResultRow) -> (&str, &str, &str, &str, &str) {
    (
        row.domain.as_str(),
        row.stage_id.as_str(),
        row.tool_id.as_str(),
        row.corpus_id.as_str(),
        row.result_id.as_str(),
    )
}

fn has_blank_or_duplicate(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().any(|value| value.trim().is_empty() || !seen.insert(value.as_str()))
}

fn unique_count<'a>(
    rows: &'a [AllDomainExpectedBenchmarkResultRow],
    key: impl Fn(&'a AllDomainExpectedBenchmarkResultRow) -> &'a str,
) -> usize {
    rows.iter().map(key).collect::<BTreeSet<_>>().len()
}

fn count_by<'a>(
    rows: &'a [AllDomainExpectedBenchmarkResultRow],
    key: impl Fn(&'a AllDomainExpectedBenchmarkResultRow) -> &'a str,
) -> BTreeMap<String, usize> {
    rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(key(row).to_string()).or_default() += 1;
        counts
    })
}

fn safe_relative_path(value: &str) -> bool {
    let path = std::path::Path::new(value);
    !value.trim().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| {
            matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir)
        })
}

fn invalid<T>(reason: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidObservation { reason: reason.to_string() })
}
