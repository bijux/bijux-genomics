//! FASTQ-to-BAM cross-domain profiles.

mod merged_defaults;
mod profiles;
mod required_stages;
mod source_profiles;

pub use profiles::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
