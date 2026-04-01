use crate::bam::bam_adna_shotgun_profile;
use crate::fastq::fastq_adna_profile;
use crate::PipelineProfile;

pub(super) fn source_profiles() -> (PipelineProfile, PipelineProfile) {
    (fastq_adna_profile(), bam_adna_shotgun_profile())
}
