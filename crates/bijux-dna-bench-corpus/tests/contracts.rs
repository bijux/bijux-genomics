#![allow(clippy::expect_used, clippy::too_many_lines, clippy::float_cmp)]
//! Existing corpus observations and rejection cases, grouped by scientific input family.
#[path = "contracts/amplicon.rs"]
mod amplicon;
#[cfg(feature = "alignment")]
#[path = "contracts/bam.rs"]
mod bam;
#[path = "contracts/damage.rs"]
mod damage;
#[path = "contracts/edna.rs"]
mod edna;
#[path = "contracts/fastq.rs"]
mod fastq;
mod support;
#[cfg(feature = "variants")]
#[path = "contracts/vcf.rs"]
mod vcf;
