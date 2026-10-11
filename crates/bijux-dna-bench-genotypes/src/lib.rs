//! Owner: bijux-dna-bench-genotypes
//! Admitted genotype calling and likelihood propagation benchmark services.
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::expect_used))]
mod bam;
pub mod call;
pub mod call_diploid;
pub mod call_gl;
pub mod call_pseudohaploid;
pub mod gl_propagation;
pub mod inputs;
mod publication;
mod stage_result;
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;

#[cfg(test)]
#[path = "tests/admission.rs"]
mod admission_tests;
#[cfg(test)]
#[path = "tests/publication.rs"]
mod publication_tests;

pub mod readiness;
#[cfg(test)]
#[path = "tests/readiness.rs"]
mod readiness_tests;
