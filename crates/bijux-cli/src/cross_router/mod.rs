//! Cross-domain pipeline runner entrypoints.

pub(crate) const CROSS_STAGE_ID: &str = "cross.align_stub";

mod bam_exec;
mod fastq_to_bam;
mod manifests;

pub use fastq_to_bam::run_fastq_to_bam_profile;
