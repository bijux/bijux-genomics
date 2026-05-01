use crate::cross::{
    bam_to_vcf_default_profile, fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile,
    fastq_to_vcf_minimal_profile,
};
use crate::PipelineProfile;

#[must_use]
pub fn cross_profiles() -> Vec<PipelineProfile> {
    vec![
        bam_to_vcf_default_profile(),
        fastq_to_bam_adna_shotgun_profile(),
        fastq_to_bam_default_profile(),
        fastq_to_vcf_minimal_profile(),
    ]
}
