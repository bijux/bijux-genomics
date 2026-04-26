use crate::bam::{bam_adna_shotgun_profile, bam_default_profile};
use crate::fastq::{fastq_adna_profile, fastq_default_profile};
use crate::PipelineProfile;

pub(super) fn default_source_profiles() -> (PipelineProfile, PipelineProfile) {
    (fastq_default_profile(), bam_default_profile())
}

pub(super) fn adna_source_profiles() -> (PipelineProfile, PipelineProfile) {
    (fastq_adna_profile(), bam_adna_shotgun_profile())
}
