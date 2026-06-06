//! Domain-owned raw parser surfaces for governed VCF stage artifacts.

mod angsd;
mod bcftools;

pub use angsd::parse_angsd_stage_metrics;
pub use bcftools::parse_bcftools_stage_metrics;
