//! Owner: complete admission of illustrative missing-result audit reports.
use crate::{
    AllDomainMissingResultRow, AllDomainMissingResultStatus, AllDomainMissingResultTestReport,
    ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION, BAM_REMOVED_RESULT_ID, FASTQ_REMOVED_RESULT_ID,
    VCF_REMOVED_RESULT_ID,
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Compute the behavior verdict only after complete report admission.
/// # Errors
/// Rejects contradictory rows, counters, removal identities and declared paths.
pub fn admit_all_domain_missing_result_audit(
    mut report: AllDomainMissingResultTestReport,
) -> Result<AllDomainMissingResultTestReport> {
    validate_content(&report)?;
    report.passes_behavior_test = true;
    Ok(report)
}

/// Admit a replay without treating its declared verdict as independent execution evidence.
/// # Errors
/// Rejects an unsuccessful declared behavior test or contradictory report content.
pub fn validate_all_domain_missing_result_audit(
    report: &AllDomainMissingResultTestReport,
) -> Result<()> {
    ensure!(report.passes_behavior_test, "missing-result behavior test has not passed");
    validate_content(report)
}

fn validate_content(report: &AllDomainMissingResultTestReport) -> Result<()> {
    ensure!(
        report.schema_version == ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION,
        "unsupported missing-result audit schema"
    );
    require_path(&report.output_path)?;
    require_path(&report.fake_result_root)?;
    ensure!(
        report.expected_row_count > 0 && report.expected_row_count == report.rows.len(),
        "missing-result row count differs from the declared complete set"
    );
    let mut by_id = BTreeMap::new();
    let mut paths = BTreeSet::new();
    let mut domains = BTreeMap::new();
    let mut missing = BTreeMap::new();
    for row in &report.rows {
        validate_row(row, &report.fake_result_root)?;
        ensure!(
            by_id.insert(row.result_id.as_str(), row).is_none()
                && paths.insert(&row.expected_manifest_path),
            "missing-result audit repeats a result identity or path"
        );
        *domains.entry(row.domain.clone()).or_insert(0_usize) += 1;
        if row.result_status == AllDomainMissingResultStatus::MissingResult {
            missing.insert(row.result_id.as_str(), row);
        }
    }
    ensure!(domains == report.domain_counts, "missing-result domain totals differ from rows");
    ensure!(
        report.missing_result_row_count == missing.len()
            && report.present_result_row_count == report.rows.len() - missing.len(),
        "missing-result status totals differ from rows"
    );
    let governed = [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID]
        .into_iter()
        .collect::<BTreeSet<_>>();
    ensure!(
        missing.keys().copied().collect::<BTreeSet<_>>() == governed,
        "missing-result audit must retain its governed FASTQ BAM and VCF omissions"
    );
    ensure!(
        missing.values().map(|row| row.domain.as_str()).collect::<BTreeSet<_>>()
            == BTreeSet::from(["bam", "fastq", "vcf"]),
        "missing-result omissions must cover one row per domain"
    );
    ensure!(
        report.removed_result_ids.len() == missing.len()
            && report.removed_manifest_paths.len() == missing.len(),
        "missing-result removals require exactly one identity and path per missing row"
    );
    let mut removed = BTreeSet::new();
    for (id, path) in report.removed_result_ids.iter().zip(&report.removed_manifest_paths) {
        ensure!(removed.insert(id.as_str()), "missing-result removal identity is repeated");
        let row = missing.get(id.as_str()).context("removed result is not classified missing")?;
        ensure!(
            path == &row.expected_manifest_path,
            "removed manifest path differs from its result identity"
        );
    }
    Ok(())
}

fn validate_row(row: &AllDomainMissingResultRow, root: &str) -> Result<()> {
    for value in [
        &row.result_id,
        &row.domain,
        &row.stage_id,
        &row.tool_id,
        &row.corpus_id,
        &row.asset_profile_id,
        &row.report_section,
        &row.reason,
    ] {
        ensure!(!value.trim().is_empty(), "missing-result row contains a blank required field");
    }
    for component in
        [&row.domain, &row.corpus_id, &row.stage_id, &row.asset_profile_id, &row.tool_id]
    {
        super::admit_path_component(component)?;
    }
    require_path(&row.expected_manifest_path)?;
    require_path(&row.audit_manifest_path)?;
    let expected_path = Path::new(root)
        .join(&row.domain)
        .join(&row.corpus_id)
        .join(&row.stage_id)
        .join(&row.asset_profile_id)
        .join(&row.tool_id)
        .join("stage-result.json");
    ensure!(
        Path::new(&row.expected_manifest_path) == expected_path
            && row.audit_manifest_path == row.expected_manifest_path,
        "missing-result manifest path is not bound to its source identity"
    );
    let expected = unique_values(&row.expected_output_artifact_ids)?;
    unique_values(&row.expected_metrics)?;
    match row.result_status {
        AllDomainMissingResultStatus::Present => ensure!(
            unique_values(&row.observed_output_artifact_ids)? == expected,
            "present result output identities differ from its declaration"
        ),
        AllDomainMissingResultStatus::MissingResult => ensure!(
            row.observed_output_artifact_ids.is_empty(),
            "missing result cannot claim observed outputs"
        ),
    }
    Ok(())
}
fn unique_values(values: &[String]) -> Result<BTreeSet<&str>> {
    let unique = values.iter().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(
        !unique.is_empty()
            && unique.len() == values.len()
            && unique.iter().all(|value| !value.trim().is_empty()),
        "missing-result declaration contains empty or duplicate values"
    );
    Ok(unique)
}
fn require_path(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && !Path::new(value).is_absolute()
            && !value.contains(['\\', '\0'])
            && value.split('/').all(|part| !part.is_empty() && part != "." && part != ".."),
        "missing-result evidence requires a normalized repository-relative path"
    );
    Ok(())
}

mod projection;
pub use projection::{
    project_all_domain_missing_result_observation, AllDomainMissingResultObservation,
};
