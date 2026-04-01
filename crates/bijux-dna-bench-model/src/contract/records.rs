//! Owner: bijux-dna-bench
//! Validation for non-suite benchmark contract records.

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkObservation, BenchmarkSummary};
use crate::policy::GateDecision;

use super::{DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUMMARY_SCHEMA_V1};

/// # Errors
/// Returns an error if required confounders are missing.
pub fn validate_observation(obs: &BenchmarkObservation) -> Result<(), BenchError> {
    if obs.schema_version != OBSERVATION_SCHEMA_V1 {
        return Err(BenchError::InvalidObservation {
            reason: format!("observation schema mismatch: {}", obs.schema_version),
        });
    }
    obs.validate()?;
    Ok(())
}

/// # Errors
/// Returns an error if summary schema is invalid.
pub fn validate_summary(summary: &BenchmarkSummary) -> Result<(), BenchError> {
    if summary.schema_version != SUMMARY_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "summary schema mismatch: {}",
            summary.schema_version
        )));
    }
    Ok(())
}

/// # Errors
/// Returns an error if decision schema is invalid.
pub fn validate_decision(decision: &GateDecision) -> Result<(), BenchError> {
    if decision.schema_version != DECISION_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "decision schema mismatch: {}",
            decision.schema_version
        )));
    }
    Ok(())
}
