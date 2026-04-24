use crate::EffectiveDefaults;

use super::source_profiles::source_profiles;

pub(super) fn base_defaults() -> (crate::PipelineProfile, crate::PipelineProfile, EffectiveDefaults)
{
    let (fastq_profile, bam_profile) = source_profiles();

    let mut defaults = EffectiveDefaults::default();
    defaults.tools.extend(fastq_profile.defaults.tools.clone());
    defaults.params.extend(fastq_profile.defaults.params.clone());
    defaults.rationales.extend(fastq_profile.defaults.rationales.clone());
    defaults.tools.extend(bam_profile.defaults.tools.clone());
    defaults.params.extend(bam_profile.defaults.params.clone());
    defaults.rationales.extend(bam_profile.defaults.rationales.clone());
    (fastq_profile, bam_profile, defaults)
}
