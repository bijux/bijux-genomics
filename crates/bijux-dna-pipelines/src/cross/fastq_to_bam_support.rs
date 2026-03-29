//! Shared support for FASTQ-to-BAM cross-domain profiles.

use bijux_dna_core::prelude::id_catalog;

use crate::bam::bam_adna_shotgun_profile;
use crate::fastq::fastq_adna_profile;
use crate::{EffectiveDefaults, PipelineProfile};

pub(super) fn base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
    let fastq_profile = fastq_adna_profile();
    let bam_profile = bam_adna_shotgun_profile();

    let mut defaults = EffectiveDefaults::default();
    defaults.tools.extend(fastq_profile.defaults.tools.clone());
    defaults
        .params
        .extend(fastq_profile.defaults.params.clone());
    defaults
        .rationales
        .extend(fastq_profile.defaults.rationales.clone());
    defaults.tools.extend(bam_profile.defaults.tools.clone());
    defaults.params.extend(bam_profile.defaults.params.clone());
    defaults
        .rationales
        .extend(bam_profile.defaults.rationales.clone());
    (fastq_profile, bam_profile, defaults)
}

pub(super) fn required_cross_stages(fastq_profile: &PipelineProfile) -> Vec<String> {
    let mut stages = fastq_profile.capabilities.required_stages.clone();
    stages.extend([
        id_catalog::CORE_PREPARE_REFERENCE.to_string(),
        id_catalog::BAM_ALIGN.to_string(),
        "bam.qc_pre".to_string(),
        id_catalog::BAM_COVERAGE.to_string(),
        id_catalog::BAM_DAMAGE.to_string(),
    ]);
    stages
}
