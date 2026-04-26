use crate::EffectiveDefaults;

use super::source_profiles::{adna_source_profiles, default_source_profiles};

fn merge_defaults(
    fastq_profile: crate::PipelineProfile,
    bam_profile: crate::PipelineProfile,
) -> (crate::PipelineProfile, crate::PipelineProfile, EffectiveDefaults) {
    let mut defaults = EffectiveDefaults::default();
    defaults.tools.extend(fastq_profile.defaults.tools.clone());
    defaults.params.extend(fastq_profile.defaults.params.clone());
    defaults.rationales.extend(fastq_profile.defaults.rationales.clone());
    defaults.tools.extend(bam_profile.defaults.tools.clone());
    defaults.params.extend(bam_profile.defaults.params.clone());
    defaults.rationales.extend(bam_profile.defaults.rationales.clone());
    (fastq_profile, bam_profile, defaults)
}

pub(super) fn default_base_defaults(
) -> (crate::PipelineProfile, crate::PipelineProfile, EffectiveDefaults) {
    let (fastq_profile, bam_profile) = default_source_profiles();
    merge_defaults(fastq_profile, bam_profile)
}

pub(super) fn adna_base_defaults(
) -> (crate::PipelineProfile, crate::PipelineProfile, EffectiveDefaults) {
    let (fastq_profile, bam_profile) = adna_source_profiles();
    merge_defaults(fastq_profile, bam_profile)
}
