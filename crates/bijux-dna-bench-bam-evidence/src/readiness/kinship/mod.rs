mod declarations;
#[cfg(feature = "bam_downstream")]
mod input;
mod models;
mod observations;
mod projection;
#[cfg(feature = "bam_downstream")]
mod proofs;
mod publication;
mod service;
pub use models::{BamKinshipReadyReport, BamKinshipReadyRow};
pub use publication::render_bam_kinship_ready;
mod checks;
mod explanation;
