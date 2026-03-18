//! Cross-domain pipeline runner entrypoints.

mod bam_exec;
mod fastq_to_bam;
mod manifests;

pub(crate) use crate::internal::alignment_boundary::AlignmentBoundary;
pub use fastq_to_bam::run_fastq_to_bam_profile;
