//! Admission invariants for persisted scientific acceptance threshold contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, ScientificAcceptanceThresholdRow, ScientificAcceptanceThresholdsConfig,
    ScientificAcceptanceThresholdsReport, SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1,
    SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1,
};

/// Validates persisted scientific acceptance threshold semantics.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when schema identity, row identity,
/// or tolerance semantics are invalid.
pub fn validate_scientific_acceptance_thresholds_config(
    config: &ScientificAcceptanceThresholdsConfig,
) -> Result<(), BenchError> {
    if config.schema_version != SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1 {
        return invalid("scientific acceptance threshold config schema identity is invalid");
    }
    validate_rows(&config.rows)
}

/// Validates a persisted scientific acceptance readiness projection.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when row semantics, aggregates,
/// schema identity, or the referenced configuration path are invalid.
pub fn validate_scientific_acceptance_thresholds_report(
    report: &ScientificAcceptanceThresholdsReport,
) -> Result<(), BenchError> {
    if report.schema_version != SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1 {
        return invalid("scientific acceptance threshold report schema identity is invalid");
    }
    validate_rows(&report.rows)?;
    if report.row_count != report.rows.len()
        || report.comparable_metric_count != report.rows.len()
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.direction_counts != count_by(&report.rows, |row| row.direction.as_str())
        || report.insufficiency_behavior_counts
            != count_by(&report.rows, |row| row.insufficiency_behavior.as_str())
    {
        return invalid("scientific acceptance threshold report aggregates are inconsistent");
    }
    if !is_safe_relative_path(&report.config_path) {
        return invalid("scientific acceptance threshold report config path is unsafe");
    }
    Ok(())
}

fn validate_rows(rows: &[ScientificAcceptanceThresholdRow]) -> Result<(), BenchError> {
    if rows.is_empty() {
        return invalid("scientific acceptance threshold rows are empty");
    }
    let mut identities = BTreeSet::new();
    for row in rows {
        if [
            row.domain.as_str(),
            row.stage_id.as_str(),
            row.metric_id.as_str(),
            row.metric_name.as_str(),
            row.declaration_origin.as_str(),
        ]
        .iter()
        .any(|value| value.trim().is_empty())
            || !row.tolerance_value.is_finite()
            || row.tolerance_value < 0.0
        {
            return invalid("scientific acceptance threshold row has invalid semantics");
        }
        let identity = (&row.domain, &row.stage_id, &row.metric_id);
        if !identities.insert(identity) {
            return invalid("scientific acceptance threshold row identity is duplicated");
        }
    }
    Ok(())
}

fn count_by<'a>(
    rows: &'a [ScientificAcceptanceThresholdRow],
    key: impl Fn(&'a ScientificAcceptanceThresholdRow) -> &'a str,
) -> BTreeMap<String, usize> {
    rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(key(row).to_string()).or_default() += 1;
        counts
    })
}

fn is_safe_relative_path(path: &str) -> bool {
    let candidate = std::path::Path::new(path);
    !path.trim().is_empty()
        && !candidate.is_absolute()
        && candidate.components().all(|component| {
            matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir)
        })
}

fn invalid<T>(reason: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidObservation { reason: reason.to_string() })
}
