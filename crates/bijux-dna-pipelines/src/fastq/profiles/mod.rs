//! FASTQ pipeline profile definitions.

mod adna_profile;
mod default_profile;
mod minimal_profile;
mod profile_by_id;
mod profile_contracts;
mod profile_ids;
mod reference_adna_profile;

pub use adna_profile::fastq_adna_profile;
pub use default_profile::fastq_default_profile;
pub use minimal_profile::fastq_minimal_profile;
pub use profile_by_id::fastq_profiles_by_id;
pub use profile_ids::FASTQ_PROFILE_IDS;
pub use reference_adna_profile::fastq_reference_adna_profile;
