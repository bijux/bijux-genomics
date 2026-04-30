use crate::cross::fastq_to_vcf::source_profiles::{fastq_minimal_profile, vcf_minimal_profile};
use crate::{EffectiveDefaults, PipelineProfile};

pub(super) fn minimal_base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
    let fastq_profile = fastq_minimal_profile();
    let vcf_profile = vcf_minimal_profile();
    let defaults = fastq_profile.defaults.clone();
    (fastq_profile, vcf_profile, defaults)
}
