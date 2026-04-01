//! Cross-domain FASTQ-to-BAM profile definitions.

mod ancient_dna_profile;
mod default_profile;

pub use ancient_dna_profile::fastq_to_bam_adna_shotgun_profile;
pub use default_profile::fastq_to_bam_default_profile;
