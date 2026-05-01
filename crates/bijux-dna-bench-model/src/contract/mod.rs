//! Owner: bijux-dna-bench-model
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

mod records;
mod schema_versions;
mod suite;
pub use records::{
    validate_bundle_manifest, validate_corpus_manifest, validate_decision, validate_observation,
    validate_summary,
};
pub use schema_versions::{
    BUNDLE_SCHEMA_V1, CORPUS_SCHEMA_V1, DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1,
    SUMMARY_SCHEMA_V1,
};
pub use suite::validate_suite;
