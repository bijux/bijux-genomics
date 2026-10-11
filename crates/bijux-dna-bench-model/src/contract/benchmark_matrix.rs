//! Owner: bijux-dna-bench-model
//! Admission rules for persisted benchmark-matrix reports.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkMatrixReport, BenchmarkSurfaceMatch};
use crate::BENCHMARK_MATRIX_REPORT_SCHEMA_V1;

/// Validate persisted benchmark-matrix identity and aggregate closure.
///
/// # Errors
/// Returns an error for schema, identity, row, readiness, surface, or summary drift.
pub fn validate_benchmark_matrix_report(report: &BenchmarkMatrixReport) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_MATRIX_REPORT_SCHEMA_V1 {
        return Err(invalid("benchmark matrix report schema mismatch"));
    }
    if report.campaign_id.trim().is_empty()
        || report.domain.trim().is_empty()
        || report.generated_at.trim().is_empty()
        || report.domains.is_empty()
    {
        return Err(invalid("benchmark matrix report identity is incomplete"));
    }

    let mut domains = BTreeSet::new();
    if report
        .domains
        .iter()
        .any(|domain| domain.trim().is_empty() || !domains.insert(domain.as_str()))
    {
        return Err(invalid("benchmark matrix report domains are invalid"));
    }

    let mut row_ids = BTreeSet::new();
    let mut readiness_counts = BTreeMap::new();
    let mut domain_counts = BTreeMap::new();
    for row in &report.rows {
        if row.row_id.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || !row_ids.insert(row.row_id.as_str())
            || !domains.contains(row.matrix_domain.as_str())
        {
            return Err(invalid("benchmark matrix report row identity is invalid"));
        }
        validate_surface(&row.corpus_match)?;
        validate_surface(&row.database_match)?;
        validate_surface(&row.image_match)?;
        let blocked_surfaces = [&row.corpus_match, &row.database_match, &row.image_match]
            .into_iter()
            .filter(|surface| !surface.ready)
            .count();
        let expected_readiness = match blocked_surfaces {
            0 => "ready",
            1 => "degraded",
            _ => "refuse",
        };
        if row.readiness.class != expected_readiness {
            return Err(invalid(
                "benchmark matrix report readiness disagrees with surface matches",
            ));
        }
        match row.readiness.class.as_str() {
            "ready" if row.readiness.reasons.is_empty() && row.repetitions > 0 => {}
            "degraded" if !row.readiness.reasons.is_empty() && row.repetitions > 0 => {}
            "refuse" if !row.readiness.reasons.is_empty() && row.repetitions == 0 => {}
            _ => return Err(invalid("benchmark matrix report readiness is invalid")),
        }
        *readiness_counts.entry(row.readiness.class.clone()).or_insert(0_usize) += 1;
        *domain_counts.entry(row.matrix_domain.clone()).or_insert(0_usize) += 1;
    }
    if report.summary.total_rows != report.rows.len()
        || report.summary.readiness_counts != readiness_counts
        || report.summary.domain_counts != domain_counts
    {
        return Err(invalid("benchmark matrix report summary drifted"));
    }
    Ok(())
}

fn validate_surface(surface: &BenchmarkSurfaceMatch) -> Result<(), BenchError> {
    if surface.required_profile.trim().is_empty() || surface.matched_profile.trim().is_empty() {
        Err(invalid("benchmark matrix report surface match is incomplete"))
    } else {
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
