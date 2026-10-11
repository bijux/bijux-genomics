//! Owner: bijux-dna-bench-variants
//! Variant filtering, damage exclusion, postprocessing and cohort quality benchmarks.
#![forbid(unsafe_code)]
pub mod damage_filter;
pub mod filter;
pub mod inputs;
pub mod postprocess;
mod publication;
pub mod qc;
mod stage_result;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/filter.rs"]
mod filter_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/damage_filter.rs"]
mod damage_filter_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/postprocess.rs"]
mod postprocess_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/qc.rs"]
mod qc_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "../tests/support/mod.rs"]
mod test_support;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/processing_publication.rs"]
mod processing_publication_contracts;

pub mod statistics;

pub mod readiness;
#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/readiness.rs"]
mod readiness_contracts;
