//! Owner: bijux-dna-bench-workflows
//! Local benchmark workflow composition with explicit inputs, effects and handoff reports.
#[cfg(feature = "genomics")]
pub mod adna;
pub mod amplicon;
#[cfg(feature = "genomics")]
pub mod core_germline;
mod execution;
#[cfg(feature = "genomics")]
pub mod inputs;

#[cfg(test)]
#[path = "tests/output.rs"]
mod output_tests;
