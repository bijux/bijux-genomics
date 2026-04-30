use bijux_dna_core::prelude::id_catalog;

use crate::PipelineProfile;

pub(super) fn required_cross_stages(fastq_profile: &PipelineProfile) -> Vec<String> {
    let mut stages = fastq_profile.capabilities.required_stages.clone();
    stages.extend([
        id_catalog::CORE_PREPARE_REFERENCE.to_string(),
        id_catalog::BAM_ALIGN.to_string(),
        "bam.qc_pre".to_string(),
        id_catalog::BAM_MAPPING_SUMMARY.to_string(),
        id_catalog::BAM_COVERAGE.to_string(),
        id_catalog::BAM_DAMAGE.to_string(),
    ]);
    stages
}
