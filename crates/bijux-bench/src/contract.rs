//! Owner: bijux-bench
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

use crate::error::BenchError;
use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
use crate::policy::GateDecision;

pub const SUITE_SCHEMA_V1: &str = "bijux.bench.suite.v1";
pub const OBSERVATION_SCHEMA_V1: &str = "bijux.bench.observation.v1";
pub const SUMMARY_SCHEMA_V1: &str = "bijux.bench.summary.v1";
pub const DECISION_SCHEMA_V1: &str = "bijux.bench.gate.v1";

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    if suite.schema_version != SUITE_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "suite schema mismatch: {}",
            suite.schema_version
        )));
    }
    if suite.datasets.is_empty() || suite.stages.is_empty() || suite.tools.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "suite must include datasets, stages, and tools".to_string(),
        ));
    }
    if suite.datasets.iter().any(|dataset| dataset.hash.trim().is_empty()) {
        return Err(BenchError::InvalidPolicy(
            "suite datasets must include hash".to_string(),
        ));
    }
    Ok(())
}

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
