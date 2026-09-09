//! Admission invariants for all-domain benchmark output declarations.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AllDomainOutputDeclarationRow, AllDomainOutputDeclarationStatus,
    AllDomainOutputDeclarationsReport, BenchError, ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1,
};

/// Validates the canonical all-domain output declaration rows.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, output shape, or ordering.
pub fn validate_all_domain_output_declaration_rows(
    rows: &[AllDomainOutputDeclarationRow],
) -> Result<(), BenchError> {
    if rows.is_empty() {
        return invalid("all-domain output declaration rows are empty");
    }
    let mut result_ids = BTreeSet::new();
    for row in rows {
        if !matches!(row.domain.as_str(), "bam" | "fastq" | "vcf")
            || !row.stage_id.starts_with(&format!("{}.", row.domain))
            || [
                row.result_id.as_str(),
                row.domain.as_str(),
                row.stage_id.as_str(),
                row.tool_id.as_str(),
                row.corpus_id.as_str(),
                row.asset_profile_id.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
            || !result_id_matches_protocol_identity(row)
            || !safe_relative_path(&row.manifest)
            || !row.manifest.ends_with("/stage-result.json")
            || !nonempty_unique(&row.raw_outputs)
            || !nonempty_unique(&row.normalized_metrics)
            || !optional_unique(&row.index_outputs)
            || !logs_match_manifest(row)
            || row.status != AllDomainOutputDeclarationStatus::Complete
            || !result_ids.insert(row.result_id.as_str())
        {
            return invalid("all-domain output declaration row semantics are invalid");
        }
    }
    if rows.windows(2).any(|rows| row_key(&rows[0]) >= row_key(&rows[1])) {
        return invalid("all-domain output declaration rows are not canonically ordered");
    }
    validate_required_rows(rows)
}

/// Validates a persisted all-domain output declaration report.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, path, rows, or aggregates.
pub fn validate_all_domain_output_declarations_report(
    report: &AllDomainOutputDeclarationsReport,
) -> Result<(), BenchError> {
    if report.schema_version != ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1
        || !safe_relative_path(&report.output_path)
    {
        return invalid("all-domain output declaration report identity or path is invalid");
    }
    validate_all_domain_output_declaration_rows(&report.rows)?;
    let result_ids = report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    let complete = report
        .rows
        .iter()
        .filter(|row| row.status == AllDomainOutputDeclarationStatus::Complete)
        .count();
    if report.row_count != report.rows.len()
        || report.result_id_count != result_ids.len()
        || report.complete_row_count != complete
        || report.incomplete_row_count != report.rows.len().saturating_sub(complete)
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.status_counts != count_by(&report.rows, |row| row.status.as_str())
    {
        return invalid("all-domain output declaration report aggregates are inconsistent");
    }
    Ok(())
}

fn result_id_matches_protocol_identity(row: &AllDomainOutputDeclarationRow) -> bool {
    let mut parts = row.result_id.split(':');
    parts.next() == Some(row.domain.as_str())
        && parts.next() == Some(row.corpus_id.as_str())
        && parts.next() == Some(row.stage_id.as_str())
        && parts.next().is_some_and(|profile| !profile.trim().is_empty())
        && parts.next() == Some(row.tool_id.as_str())
        && parts.next().is_none()
}

fn validate_required_rows(rows: &[AllDomainOutputDeclarationRow]) -> Result<(), BenchError> {
    require_output_row(
        rows,
        "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
        "classification_report_json",
        "screen_report_tsv",
        "artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json",
        None,
    )?;
    require_output_row(
        rows,
        "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
        "kinship_report",
        "summary",
        "artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king/stage-result.json",
        None,
    )?;
    require_output_row(
        rows,
        "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
        "called_vcf",
        "called_vcf",
        "artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json",
        Some("called_vcf_tbi"),
    )
}

fn require_output_row(
    rows: &[AllDomainOutputDeclarationRow],
    result_id: &str,
    normalized_metric: &str,
    raw_output: &str,
    manifest: &str,
    index_output: Option<&str>,
) -> Result<(), BenchError> {
    let Some(row) = rows.iter().find(|row| row.result_id == result_id) else {
        return invalid("all-domain output declarations are missing a required proof row");
    };
    if !row.normalized_metrics.iter().any(|value| value == normalized_metric)
        || !row.raw_outputs.iter().any(|value| value == raw_output)
        || row.manifest != manifest
        || index_output
            .is_some_and(|required| !row.index_outputs.iter().any(|value| value == required))
    {
        return invalid("all-domain output declaration required proof row drifted");
    }
    Ok(())
}

fn logs_match_manifest(row: &AllDomainOutputDeclarationRow) -> bool {
    let root = row.manifest.trim_end_matches("/stage-result.json");
    row.logs.len() == 2
        && row.logs.iter().collect::<BTreeSet<_>>().len() == 2
        && row.logs.iter().any(|value| value == &format!("stdout={root}/stdout.log"))
        && row.logs.iter().any(|value| value == &format!("stderr={root}/stderr.log"))
}

fn nonempty_unique(values: &[String]) -> bool {
    !values.is_empty() && optional_unique(values)
}

fn optional_unique(values: &[String]) -> bool {
    values.iter().all(|value| !value.trim().is_empty())
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn row_key(row: &AllDomainOutputDeclarationRow) -> (&str, &str, &str, &str) {
    (&row.domain, &row.stage_id, &row.tool_id, &row.result_id)
}

fn count_by<'a>(
    rows: &'a [AllDomainOutputDeclarationRow],
    key: impl Fn(&'a AllDomainOutputDeclarationRow) -> &'a str,
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
