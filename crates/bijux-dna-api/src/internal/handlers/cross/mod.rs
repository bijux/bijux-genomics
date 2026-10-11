//! Cross-domain pipeline runner entrypoints.

mod bam_exec;
mod fastq_to_bam;
mod manifests;

pub(crate) use bijux_dna_domain_bam::alignment::AlignmentBoundary;
pub use fastq_to_bam::run_fastq_to_bam_profile;
