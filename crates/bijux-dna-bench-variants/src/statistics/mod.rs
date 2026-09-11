//! Governed cohort statistics own admission, scientific observations and retained publication.
mod admission;
mod fixtures;
mod manifest;
mod metrics;
mod model;
mod timing;
mod truth;
mod workflow;
pub use model::{LocalVcfStatsSmokeMetrics, LocalVcfStatsSmokeReport};
pub use workflow::run_local_vcf_stats_smoke;
mod execution;
mod publication;
mod report;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests.rs"]
mod tests;
