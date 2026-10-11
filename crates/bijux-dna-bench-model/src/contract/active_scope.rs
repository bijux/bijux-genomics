//! Owner: bijux-dna-bench-model
//! Admission rules for active-scope validation records.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::active_scope::ACTIVE_SCOPE_FAST_VALIDATION_MODE;
use crate::model::{ActiveScopeFastValidationReport, ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1};

const REQUIRED_ACTIVE_SCOPE_FAST_SURFACES: &[(&str, &str)] = &[
    ("adapters", "all_domain_adapter_coverage"),
    ("commands", "all_domain_local_job_coverage"),
    ("commands", "all_domain_no_placeholder_command_check"),
    ("configs", "all_domain_active_stage_tool_matrix"),
    ("expected_results", "all_domain_expected_result_coverage"),
    ("fixtures", "benchmark_fixture_root_validation"),
    ("outputs", "all_domain_output_contract_coverage"),
    ("parsers", "all_domain_parser_fixture_coverage"),
    ("reports", "all_domain_report_map_coverage"),
    ("schemas", "all_domain_schema_validation"),
];

/// Validate active-scope report identity and exact count closure.
///
/// # Errors
/// Returns an error for an unsupported schema, incomplete checks, duplicate surfaces, or summary
/// drift.
pub fn validate_active_scope_fast_validation_report(
    report: &ActiveScopeFastValidationReport,
) -> Result<(), BenchError> {
    if report.schema_version != ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1
        || report.mode != ACTIVE_SCOPE_FAST_VALIDATION_MODE
        || report.checks.is_empty()
        || [
            report.output_path.as_str(),
            report.benchmark_root.as_str(),
            report.schema_root.as_str(),
            report.fixture_root.as_str(),
        ]
        .into_iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(invalid("active-scope report identity is invalid"));
    }

    let mut identities = BTreeSet::new();
    let mut category_counts = BTreeMap::new();
    let mut failed_category_counts = BTreeMap::new();
    let mut passed = 0_usize;
    for check in &report.checks {
        if [
            check.category.as_str(),
            check.surface_id.as_str(),
            check.output_path.as_str(),
            check.detail.as_str(),
        ]
        .into_iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(invalid("active-scope report check is incomplete"));
        }
        if !identities.insert((check.category.as_str(), check.surface_id.as_str())) {
            return Err(invalid("active-scope report contains a duplicate surface"));
        }
        *category_counts.entry(check.category.clone()).or_insert(0_usize) += 1;
        if check.ok {
            passed += 1;
        } else {
            *failed_category_counts.entry(check.category.clone()).or_insert(0_usize) += 1;
        }
    }
    let required_identities = REQUIRED_ACTIVE_SCOPE_FAST_SURFACES.iter().copied().collect();
    if identities != required_identities {
        return Err(invalid("active-scope report surface closure drifted"));
    }
    let failed = report.checks.len().saturating_sub(passed);
    if report.checked_surface_count != report.checks.len()
        || report.passed_surface_count != passed
        || report.failed_surface_count != failed
        || report.category_counts != category_counts
        || report.failed_category_counts != failed_category_counts
        || report.ok != (failed == 0)
    {
        return Err(invalid("active-scope report summary drifted"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
