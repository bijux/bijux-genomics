//! Semantic admission for all-domain benchmark failure classifications.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    AllDomainFailureClassificationReport, BenchError, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1,
};

pub const REQUIRED_ALL_DOMAIN_FAILURE_CLASSES: [&str; 8] = [
    "missing_input",
    "blocked_external_access",
    "tool_not_found",
    "command_failed",
    "missing_output",
    "parser_failed",
    "insufficient_data",
    "unsupported_pair",
];

/// Validate and admit an all-domain failure-classification report.
///
/// # Errors
/// Returns an error unless every governed class is uniquely triggered and counters close.
pub fn validate_all_domain_failure_classification(
    report: &mut AllDomainFailureClassificationReport,
) -> Result<(), BenchError> {
    if report.schema_version != ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1 {
        return invalid("unsupported all-domain failure-classification schema");
    }
    require_path(&report.output_path, "output_path")?;
    require_path(&report.fixture_root, "fixture_root")?;
    if report.rows.len() != REQUIRED_ALL_DOMAIN_FAILURE_CLASSES.len()
        || report.row_count != report.rows.len()
        || report.required_class_count != REQUIRED_ALL_DOMAIN_FAILURE_CLASSES.len()
    {
        return invalid("failure classification requires exactly one row per governed class");
    }
    let seen = report.rows.iter().map(|row| row.class_id.as_str()).collect::<BTreeSet<_>>();
    let required = REQUIRED_ALL_DOMAIN_FAILURE_CLASSES.into_iter().collect::<BTreeSet<_>>();
    if seen != required {
        return invalid("failure classification rows drifted from the governed class set");
    }
    for row in &report.rows {
        for value in [
            &row.class_id,
            &row.domain,
            &row.stage_id,
            &row.tool_id,
            &row.source_surface,
            &row.evidence_path,
            &row.observed_status,
            &row.detail,
        ] {
            if value.trim().is_empty() {
                return invalid("failure classification rows contain a blank required field");
            }
        }
        require_path(&row.evidence_path, "rows[].evidence_path")?;
        if row.stage_id.split_once('.').map(|(domain, _)| domain) != Some(row.domain.as_str())
            || row.result_id.as_ref().is_some_and(|value| value.trim().is_empty())
        {
            return invalid("failure classification row identity is inconsistent");
        }
    }
    if report.class_counts.len() != required.len()
        || report.class_counts.values().sum::<usize>() != report.row_count
        || report
            .class_counts
            .iter()
            .any(|(class_id, count)| !required.contains(class_id.as_str()) || *count != 1)
    {
        return invalid("failure classification class counts are inconsistent");
    }
    let triggered = report.rows.iter().filter(|row| row.triggered).count();
    let triggered_classes = report
        .rows
        .iter()
        .filter(|row| row.triggered)
        .map(|row| row.class_id.as_str())
        .collect::<BTreeSet<_>>();
    let missing = required.difference(&triggered_classes).copied().collect::<BTreeSet<_>>();
    if report.triggered_row_count != triggered
        || report.triggered_class_count != triggered_classes.len()
        || report.missing_class_count != missing.len()
        || report.missing_class_ids.iter().map(String::as_str).collect::<BTreeSet<_>>() != missing
    {
        return invalid("failure classification trigger or missing-class counters drifted");
    }
    report.passes_behavior_test = triggered == required.len() && missing.is_empty();
    if !report.passes_behavior_test {
        return invalid("failure classification must trigger every governed class");
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "failure classification requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
