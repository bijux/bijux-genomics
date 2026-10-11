//! Owner: bijux-dna-bench-population
//! Population benchmark admission, execution, normalized evidence and publication.
#![forbid(unsafe_code)]
pub mod admixture;
pub mod demography;
mod execution;
pub mod ibd;
pub mod inputs;
pub mod pca;
pub mod population_structure;
pub mod roh;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/pca.rs"]
mod pca_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/admixture.rs"]
mod admixture_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/population_structure.rs"]
mod population_structure_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/roh.rs"]
mod roh_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/ibd.rs"]
mod ibd_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/demography.rs"]
mod demography_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "../tests/support/mod.rs"]
mod test_support;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/publication.rs"]
mod publication_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/inputs.rs"]
mod input_contracts;

pub mod readiness;
#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/readiness.rs"]
mod readiness_contracts;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests/pca_readiness.rs"]
mod pca_readiness_contracts;
