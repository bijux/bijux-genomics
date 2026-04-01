use crate::fastq::{
    fastq_adna_profile, fastq_default_profile, fastq_minimal_profile, fastq_reference_adna_profile,
};
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
