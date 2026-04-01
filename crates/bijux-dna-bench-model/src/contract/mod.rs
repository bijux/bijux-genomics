//! Owner: bijux-dna-bench
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

mod edge_validation;
mod param_binding_validation;
mod records;
mod schema_versions;
mod stage_governance;
mod suite;
mod suite_validation;
pub use records::{validate_decision, validate_observation, validate_summary};
pub use schema_versions::{
    DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1,
};
pub use suite_validation::validate_suite;

#[cfg(test)]
mod suite_validation_cases;
