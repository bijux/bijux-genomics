#![allow(clippy::expect_used)]
//! Adapter family contracts share one native test target.
#[path = "contracts/angsd.rs"]
mod angsd;
#[path = "contracts/bcftools.rs"]
mod bcftools;
#[path = "contracts/descent.rs"]
mod descent;
#[path = "contracts/eigensoft.rs"]
mod eigensoft;
#[path = "contracts/imputation.rs"]
mod imputation;
#[path = "contracts/phasing.rs"]
mod phasing;
#[path = "contracts/plink.rs"]
mod plink;
mod support;
