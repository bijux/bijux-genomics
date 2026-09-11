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

#[path = "contracts/serving.rs"]
mod serving;

#[path = "contracts/observations.rs"]
mod observations;

#[path = "contracts/expectations.rs"]
mod expectations;

#[path = "contracts/output_coverage.rs"]
mod output_coverage;

#[path = "contracts/reports.rs"]
mod reports;

#[path = "contracts/scope.rs"]
mod scope;

#[path = "contracts/parser_admission.rs"]
mod parser_admission;
#[path = "contracts/parsers.rs"]
mod parsers;

#[path = "contracts/pca_evidence.rs"]
mod pca_evidence;
