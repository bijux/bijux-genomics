//! FASTQ pipeline profiles and defaults.

mod defaults;

pub mod invariants;
pub mod profiles;

pub use invariants::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};
pub use profiles::{
    fastq_adna_profile, fastq_default_profile, fastq_minimal_profile, fastq_profiles_by_id,
    fastq_reference_adna_profile, FASTQ_PROFILE_IDS,
};
