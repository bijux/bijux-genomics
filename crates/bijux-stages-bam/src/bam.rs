//! Compatibility re-exports for BAM stage planners.

pub use crate::stages_adna::{authenticity, contamination, damage, sex};
#[cfg(feature = "bam_downstream")]
pub use crate::stages_downstream::{bias_mitigation, genotyping, haplogroups, kinship};
pub use crate::stages_post::{complexity, coverage, markdup, recalibration};
pub use crate::stages_pre::{align, filter, qc_pre, validate};
