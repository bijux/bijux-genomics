//! Owner: bijux-dna-bench-model
//! Admission rules for the persisted container report portfolio.

use std::collections::BTreeSet;

use crate::diagnostics::BenchError;
use crate::model::{
    ContainerReportFreshness, ContainerReportPortfolioIndex,
    CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1,
};

/// Validate report identity, file pairs, digests, counts, and conservative freshness policy.
///
/// # Errors
/// Returns an error when a report is duplicated, incomplete, or promoted without current proof.
pub fn validate_container_report_portfolio_index(
    index: &ContainerReportPortfolioIndex,
) -> Result<(), BenchError> {
    if index.schema_version != CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1
        || index.summary.report_count != index.reports.len()
    {
        return Err(invalid("container report portfolio index summary is invalid"));
    }
    let current = index
        .reports
        .iter()
        .filter(|report| report.freshness == ContainerReportFreshness::Current)
        .count();
    if index.summary.current_count != current
        || index.summary.historical_unvalidated_count != index.reports.len() - current
    {
        return Err(invalid("container report portfolio freshness counts drifted"));
    }
    let mut report_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for report in &index.reports {
        if report.report_id.trim().is_empty()
            || !report_ids.insert(report.report_id.as_str())
            || (report.json.is_none() && report.markdown.is_none())
            || report.limitations.is_empty()
        {
            return Err(invalid("container report portfolio entry is incomplete"));
        }
        if report.freshness == ContainerReportFreshness::Current
            && report.report_id != "permanent-run-index"
        {
            return Err(invalid(
                "container report was promoted without complete current hash closure",
            ));
        }
        for file in [report.json.as_ref(), report.markdown.as_ref()].into_iter().flatten() {
            if file.sha256.len() != 64
                || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
                || !paths.insert(file.path.as_str())
            {
                return Err(invalid("container report file identity is invalid"));
            }
        }
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
