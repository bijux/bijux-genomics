//! Cross-domain pipeline profiles.

mod fastq_to_bam;
mod fastq_to_bam_support;

pub use fastq_to_bam::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
