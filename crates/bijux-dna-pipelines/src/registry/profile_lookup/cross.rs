use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::id_catalog;

use crate::cross::{
    bam_to_vcf_default_profile, fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile,
    fastq_to_vcf_minimal_profile,
};
use crate::PipelineProfile;

pub(super) fn profile_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_BAM_TO_VCF_DEFAULT => Ok(bam_to_vcf_default_profile()),
        id_catalog::PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN => Ok(fastq_to_bam_adna_shotgun_profile()),
        id_catalog::PIPELINE_FASTQ_TO_BAM_DEFAULT => Ok(fastq_to_bam_default_profile()),
        id_catalog::PIPELINE_FASTQ_TO_VCF_MINIMAL => Ok(fastq_to_vcf_minimal_profile()),
        _ => Err(anyhow!("unknown cross-domain profile: {id}")),
    }
}
