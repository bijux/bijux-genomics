use crate::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use crate::PipelineProfile;

#[must_use]
pub fn cross_profiles() -> Vec<PipelineProfile> {
    vec![
        fastq_to_bam_adna_shotgun_profile(),
        fastq_to_bam_default_profile(),
    ]
}
