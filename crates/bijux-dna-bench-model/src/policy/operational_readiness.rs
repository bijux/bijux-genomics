//! Summarize explicit operational readiness observations without acquiring or publishing artifacts.

use crate::{
    OperationalBenchmarkReadyBlocker, OperationalBenchmarkReadyCheck,
    OperationalBenchmarkReadyReport, OperationalBenchmarkResultCounts,
    OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

/// Construct the stable readiness summary from observed checks and unique blockers.
/// These observations describe local readiness, not independent backend execution.
#[must_use]
pub fn build_operational_benchmark_ready_report(
    output_path: String,
    checks: Vec<OperationalBenchmarkReadyCheck>,
    blockers: BTreeSet<OperationalBenchmarkReadyBlocker>,
    counts: OperationalBenchmarkResultCounts,
) -> OperationalBenchmarkReadyReport {
    let passed_surface_count = checks.iter().filter(|check| check.ok).count();
    let failed_surface_count = checks.len() - passed_surface_count;
    OperationalBenchmarkReadyReport {
        schema_version: OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION.to_string(),
        output_path,
        checked_surface_count: checks.len(),
        passed_surface_count,
        failed_surface_count,
        benchmark_ready_row_count: counts.benchmark_ready,
        blocker_count: blockers.len(),
        missing_result_row_count: counts.missing,
        insufficient_data_row_count: counts.insufficient,
        unsupported_pair_row_count: counts.unsupported,
        ok: blockers.is_empty() && failed_surface_count == 0,
        checks,
        blockers: blockers.into_iter().collect(),
    }
}

/// Validate summary closure when replaying a persisted readiness report.
///
/// # Errors
/// Returns an invalid observation when version, counts, ordering or the decision disagree.
pub fn validate_operational_benchmark_ready_report(
    report: &OperationalBenchmarkReadyReport,
) -> Result<(), crate::BenchError> {
    let passed = report.checks.iter().filter(|check| check.ok).count();
    let failed = report.checks.len() - passed;
    if report.schema_version != OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION
        || report.checked_surface_count != report.checks.len()
        || report.passed_surface_count != passed
        || report.failed_surface_count != failed
        || report.blocker_count != report.blockers.len()
        || report.ok != (report.blockers.is_empty() && failed == 0)
        || report.blockers.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(crate::BenchError::InvalidObservation {
            reason: "operational readiness summary disagrees with its observations".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operational_readiness_preserves_failures_and_observed_count_categories() -> anyhow::Result<()>
    {
        let checks = vec![OperationalBenchmarkReadyCheck {
            surface_id: "adapter".into(),
            output_path: "evidence.json".into(),
            ok: false,
            detail: "source unavailable".into(),
        }];
        let report = build_operational_benchmark_ready_report(
            "report.json".into(),
            checks,
            BTreeSet::new(),
            OperationalBenchmarkResultCounts {
                benchmark_ready: 7,
                missing: 2,
                insufficient: 3,
                unsupported: 1,
            },
        );
        assert!(!report.ok);
        assert_eq!(
            (
                report.checked_surface_count,
                report.failed_surface_count,
                report.passed_surface_count
            ),
            (1, 1, 0)
        );
        assert_eq!(
            (
                report.benchmark_ready_row_count,
                report.missing_result_row_count,
                report.insufficient_data_row_count,
                report.unsupported_pair_row_count
            ),
            (7, 2, 3, 1)
        );
        validate_operational_benchmark_ready_report(&report)?;
        let mut changed = report.clone();
        changed.ok = true;
        assert!(validate_operational_benchmark_ready_report(&changed).is_err());
        changed = report.clone();
        changed.checked_surface_count += 1;
        assert!(validate_operational_benchmark_ready_report(&changed).is_err());
        let wire = serde_json::to_value(&report)?;
        let replay: OperationalBenchmarkReadyReport = serde_json::from_value(wire.clone())?;
        assert_eq!(serde_json::to_value(replay)?, wire);
        for (field, value) in [
            ("schema_version", serde_json::json!("unsupported.v1")),
            ("checked_surface_count", serde_json::json!(-1)),
            ("unknown", serde_json::json!(true)),
        ] {
            let mut invalid = wire.clone();
            invalid[field] = value;
            assert!(serde_json::from_value::<OperationalBenchmarkReadyReport>(invalid).is_err());
        }
        let schema = serde_json::to_value(schemars::schema_for!(OperationalBenchmarkReadyReport))?;
        assert_eq!(
            schema["properties"]["schema_version"]["const"],
            OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION
        );
        assert_eq!(
            schema["$defs"]["OperationalBenchmarkReadyCheck"]["additionalProperties"],
            false
        );
        assert_eq!(
            schema["$defs"]["OperationalBenchmarkReadyBlocker"]["additionalProperties"],
            false
        );
        Ok(())
    }
}
