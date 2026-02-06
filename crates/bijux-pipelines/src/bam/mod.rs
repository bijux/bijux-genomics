//! BAM pipeline profiles and presets.

mod invariants;
mod profiles;

pub use invariants::{adna_invariants, DamageExpectation, DamageExpectationModel};
pub use profiles::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile, bam_profiles_by_id,
};
