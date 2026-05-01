//! BAM pipeline profiles and presets.

mod adna_profiles;
mod baseline_profiles;
mod invariants;
mod profile_capabilities;
mod profile_defaults;
mod profile_invariants;
mod profiles;
mod workflow_registry;

pub use invariants::{adna_invariants, DamageExpectation, DamageExpectationModel};
pub use profile_invariants::{
    validate_bam_profile, BamProfileValidationReport, BamProfileViolation, BAM_INVARIANTS,
};
pub use profiles::{
    bam_adna_capture_profile, bam_adna_profile, bam_adna_shotgun_profile, bam_default_profile,
    bam_profiles_by_id, bam_reference_adna_profile,
};
pub use workflow_registry::{
    bam_workflow_template_by_id, bam_workflow_templates, bam_workflow_templates_for_pipeline,
};
