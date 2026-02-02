//! Pipeline profiles and aDNA-specific presets for BAM.

mod adna_invariants;
mod profiles;

pub use adna_invariants::{adna_invariants, DamageExpectation, DamageExpectationModel};
pub use profiles::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile, profile_by_id,
    BamPipelineProfile, BamStageDefaults,
};
