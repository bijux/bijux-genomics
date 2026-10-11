//! Owner: bijux-dna-bench-haplotypes
//! Haplotype benchmark inputs, scientific execution and artifact publication.
#![forbid(unsafe_code)]
pub mod imputation;
pub mod imputation_metrics;
pub mod inputs;
mod panel_contracts;
pub mod phasing;
mod publication;
pub mod reference_panel;
mod stage_result;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/publication.rs"]
mod publication_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/phasing.rs"]
mod phasing_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/imputation.rs"]
mod imputation_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/imputation_metrics.rs"]
mod imputation_metrics_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/reference_panel.rs"]
mod reference_panel_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/panel_contracts.rs"]
mod panel_contracts_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "../tests/support/mod.rs"]
mod test_support;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/inputs.rs"]
mod input_contracts;

pub mod readiness;
#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/readiness.rs"]
mod readiness_contracts;
