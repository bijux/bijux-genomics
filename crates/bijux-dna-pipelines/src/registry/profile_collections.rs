use crate::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
    bam_reference_adna_profile,
};
use crate::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use crate::fastq::{
    fastq_adna_profile, fastq_default_profile, fastq_minimal_profile, fastq_reference_adna_profile,
};
use crate::vcf::{vcf_minimal_profile, vcf_reference_basic_profile};
use crate::PipelineProfile;

#[must_use]
pub fn fastq_profiles() -> Vec<PipelineProfile> {
    vec![
        fastq_default_profile(),
        fastq_minimal_profile(),
        fastq_adna_profile(),
        fastq_reference_adna_profile(),
    ]
}

#[must_use]
pub fn bam_profiles() -> Vec<PipelineProfile> {
    vec![
        bam_default_profile(),
        bam_adna_shotgun_profile(),
        bam_adna_capture_profile(),
        bam_reference_adna_profile(),
    ]
}

#[must_use]
pub fn cross_profiles() -> Vec<PipelineProfile> {
    vec![
        fastq_to_bam_adna_shotgun_profile(),
        fastq_to_bam_default_profile(),
    ]
}

#[must_use]
pub fn vcf_profiles() -> Vec<PipelineProfile> {
    vec![vcf_minimal_profile(), vcf_reference_basic_profile()]
}
