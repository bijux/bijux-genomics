use crate::cross::bam_to_vcf::source_profiles::{bam_default_profile, vcf_minimal_profile};
use crate::{EffectiveDefaults, PipelineProfile};

pub(super) fn default_base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
    let bam_profile = bam_default_profile();
    let vcf_profile = vcf_minimal_profile();
    let defaults = bam_profile.defaults.clone();
    (bam_profile, vcf_profile, defaults)
}
