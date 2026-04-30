use crate::cross::fastq_to_vcf::source_profiles::{fastq_minimal_profile, vcf_minimal_profile};
use crate::defaults::merge_effective_defaults;
use crate::{EffectiveDefaults, PipelineProfile};

pub(super) fn minimal_base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
    let fastq_profile = fastq_minimal_profile();
    let vcf_profile = vcf_minimal_profile();
    let defaults = merge_effective_defaults(
        &fastq_profile.defaults,
        Some(&vcf_profile.defaults),
        None,
        None,
    )
    .expect("cross FASTQ-to-VCF defaults must merge");
    (fastq_profile, vcf_profile, defaults)
}
