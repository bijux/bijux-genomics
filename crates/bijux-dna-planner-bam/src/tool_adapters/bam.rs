//! Compatibility re-exports for BAM stage planners.

pub use super::stages_adna::{authenticity, contamination, damage, sex};
#[cfg(feature = "bam_downstream")]
pub use super::stages_downstream::{bias_mitigation, genotyping, haplogroups, kinship};
pub use super::stages_post::{complexity, coverage, gc_bias, insert_size, markdup, recalibration};
pub use super::stages_pre::{align, filter, qc_pre, validate};
