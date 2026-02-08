//! Owner: bijux-benchmark
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

use crate::error::BenchError;
use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
use crate::policy::GateDecision;

mod schemas;
pub use schemas::{DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1};

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
    if suite
        .datasets
        .iter()
        .any(|dataset| dataset.hash.trim().is_empty())
    {
        return Err(BenchError::InvalidPolicy(
            "suite datasets must include hash".to_string(),
        ));
    }
    if suite.datasets.len() < suite.diversity.min_dataset_count {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} datasets",
            suite.diversity.min_dataset_count
        )));
    }
    let mut classes = std::collections::BTreeSet::new();
    let mut layouts = std::collections::BTreeSet::new();
    for dataset in &suite.datasets {
        classes.insert(dataset.class_label.as_str());
        layouts.insert(dataset.read_layout.as_str());
    }
    if classes.len() < suite.diversity.min_classes {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} dataset classes",
            suite.diversity.min_classes
        )));
    }
    if layouts.len() < suite.diversity.min_read_layouts {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} read layouts",
            suite.diversity.min_read_layouts
        )));
    }
    for requirement in &suite.stratifications {
        let values: std::collections::BTreeSet<&str> = match requirement.key.as_str() {
            "dataset_class" => suite
                .datasets
                .iter()
                .map(|dataset| dataset.class_label.as_str())
                .collect(),
            "read_layout" => suite
                .datasets
                .iter()
                .map(|dataset| dataset.read_layout.as_str())
                .collect(),
            _ => {
                return Err(BenchError::InvalidPolicy(format!(
                    "unsupported stratification key {}",
                    requirement.key
                )))
            }
        };
        for required in &requirement.required_values {
            if !values.contains(required.as_str()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite missing required stratification value {} for {}",
                    required, requirement.key
                )));
            }
        }
    }
    if suite.analysis_requirements.require_bootstrap
        && suite.replicate_policy.count < suite.analysis_requirements.min_replicates_for_bootstrap
    {
        return Err(BenchError::InvalidPolicy(format!(
            "suite requires bootstrap with at least {} replicates",
            suite.analysis_requirements.min_replicates_for_bootstrap
        )));
    }
    if suite.analysis_requirements.require_outlier_detection && suite.replicate_policy.count < 3 {
        return Err(BenchError::InvalidPolicy(
            "suite requires outlier detection with at least 3 replicates".to_string(),
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
