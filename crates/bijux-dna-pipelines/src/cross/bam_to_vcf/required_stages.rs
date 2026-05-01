use bijux_dna_core::prelude::id_catalog;

use crate::PipelineProfile;

pub(super) fn required_cross_stages(bam_profile: &PipelineProfile) -> Vec<String> {
    let mut stages = bam_profile.capabilities.required_stages.clone();
    stages.insert(0, id_catalog::CORE_PREPARE_REFERENCE.to_string());
    stages.extend([
        "bam.genotyping".to_string(),
        "vcf.filter".to_string(),
        "vcf.stats".to_string(),
    ]);
    stages
}
