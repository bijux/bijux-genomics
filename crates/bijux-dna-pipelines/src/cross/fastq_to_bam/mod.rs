//! FASTQ-to-BAM cross-domain profiles.

mod defaults;
mod profiles;
mod required_stages;

pub use profiles::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
