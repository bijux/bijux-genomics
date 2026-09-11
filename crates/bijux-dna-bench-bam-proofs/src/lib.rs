//! Scientific BAM benchmark output admission, completion assessment and publication.
mod context;
pub use context::ProofContext;
pub mod authenticity;
pub mod contamination;
pub mod damage;
pub mod endogenous_content;
pub mod haplogroups;
pub mod overlap_correction;
pub mod ports;
pub mod recalibration;
pub mod sex;

pub mod downstream;
pub mod genotyping;
mod input;
pub mod kinship;
