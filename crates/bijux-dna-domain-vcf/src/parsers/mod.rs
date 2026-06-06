//! Domain-owned raw parser surfaces for governed VCF stage artifacts.

mod angsd;
mod bcftools;
mod plink_family;

pub use angsd::parse_angsd_stage_metrics;
pub use bcftools::parse_bcftools_stage_metrics;
pub use plink_family::{parse_plink2_stage_metrics, parse_plink_stage_metrics};
