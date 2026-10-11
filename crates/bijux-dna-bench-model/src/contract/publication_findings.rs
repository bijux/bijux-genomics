//! Semantic validation for persisted supplemental publication findings.

use std::collections::BTreeSet;

use crate::{
    validate_utc_timestamp, BenchError, BenchmarkPublicationFindings,
    BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1,
};

/// Validate supplemental publication finding identity, freshness, and issue rows.
///
/// # Errors
/// Returns an error when the document is unversioned, stale-shaped, or internally inconsistent.
pub fn validate_benchmark_publication_findings(
    report: &BenchmarkPublicationFindings,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1 {
        return invalid("unsupported benchmark publication findings schema");
    }
    require_text(&report.corpus_id, "corpus_id")?;
    validate_utc_timestamp(&report.generated_at_utc, "generated_at_utc")?;

    let mut identities = BTreeSet::new();
    for finding in &report.findings {
        require_text(&finding.stage_id, "findings[].stage_id")?;
        require_text(&finding.issue_id, "findings[].issue_id")?;
        require_text(&finding.detail, "findings[].detail")?;
        if !matches!(finding.severity.as_str(), "error" | "warning") {
            return invalid(format!(
                "publication finding `{}/{}` has unknown severity `{}`",
                finding.stage_id, finding.issue_id, finding.severity
            ));
        }
        if !identities.insert((finding.stage_id.as_str(), finding.issue_id.as_str())) {
            return invalid(format!(
                "publication findings repeat `{}/{}`",
                finding.stage_id, finding.issue_id
            ));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("benchmark publication findings require non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
