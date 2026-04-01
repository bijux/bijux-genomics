use crate::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
    bam_reference_adna_profile,
};
use crate::PipelineProfile;

#[must_use]
pub fn bam_profiles() -> Vec<PipelineProfile> {
    vec![
        bam_default_profile(),
        bam_adna_shotgun_profile(),
        bam_adna_capture_profile(),
        bam_reference_adna_profile(),
    ]
}
