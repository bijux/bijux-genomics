//! FASTQ pipeline profile definitions.

mod ancient_dna_profiles;
mod baseline_profiles;
mod catalog;
mod contract_templates;

pub use ancient_dna_profiles::{fastq_adna_profile, fastq_reference_adna_profile};
pub use baseline_profiles::{fastq_default_profile, fastq_minimal_profile};
pub use catalog::{fastq_profiles_by_id, FASTQ_PROFILE_IDS};
