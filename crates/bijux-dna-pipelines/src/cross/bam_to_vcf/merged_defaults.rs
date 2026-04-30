use crate::cross::bam_to_vcf::source_profiles::{bam_default_profile, vcf_minimal_profile};
use crate::defaults::merge_effective_defaults;
use crate::{EffectiveDefaults, PipelineProfile};

pub(super) fn default_base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
    let bam_profile = bam_default_profile();
    let vcf_profile = vcf_minimal_profile();
    let defaults = merge_effective_defaults(
        &bam_profile.defaults,
        Some(&vcf_profile.defaults),
        None,
        None,
    )
    .expect("cross BAM-to-VCF defaults must merge");
    (bam_profile, vcf_profile, defaults)
}
