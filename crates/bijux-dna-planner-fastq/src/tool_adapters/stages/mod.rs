//! Stage adapter groupings for FASTQ planning.

pub mod catalog;
pub mod pre;
pub mod qc;
pub mod transform;

pub use catalog::STAGES_NAMESPACE;
