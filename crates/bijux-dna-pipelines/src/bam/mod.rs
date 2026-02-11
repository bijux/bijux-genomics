//! BAM pipeline profiles and presets.

mod invariants;
mod profile_invariants;
mod profiles;

pub use invariants::{adna_invariants, DamageExpectation, DamageExpectationModel};
pub use profile_invariants::{
    validate_bam_profile, BamProfileValidationReport, BamProfileViolation, BAM_INVARIANTS,
};
pub use profiles::{
    bam_adna_capture_profile, bam_adna_profile, bam_adna_shotgun_profile, bam_default_profile,
    bam_profiles_by_id,
};
