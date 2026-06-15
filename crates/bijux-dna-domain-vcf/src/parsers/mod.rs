//! Domain-owned raw parser surfaces for governed VCF stage artifacts.

mod angsd;
mod bcftools;
mod eigensoft;
mod imputation;
mod phasing;
mod plink_family;
mod segments;

pub use angsd::parse_angsd_stage_metrics;
pub use bcftools::parse_bcftools_stage_metrics;
pub use eigensoft::parse_eigensoft_stage_metrics;
pub use imputation::parse_imputation_stage_metrics;
pub use phasing::parse_phasing_stage_metrics;
pub use plink_family::{parse_plink2_stage_metrics, parse_plink_stage_metrics};
pub use segments::parse_segment_stage_metrics;
