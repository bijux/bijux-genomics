//! FASTQ pipeline profile definitions.

mod adna_profile;
mod contract_templates;
mod default_profile;
mod minimal_profile;
mod profile_lookup;
mod reference_adna_profile;

pub use adna_profile::fastq_adna_profile;
pub use default_profile::fastq_default_profile;
pub use minimal_profile::fastq_minimal_profile;
pub use profile_lookup::{fastq_profiles_by_id, FASTQ_PROFILE_IDS};
pub use reference_adna_profile::fastq_reference_adna_profile;
