use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::id_catalog;

use crate::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use crate::PipelineProfile;

pub(super) fn profile_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN => Ok(fastq_to_bam_adna_shotgun_profile()),
        id_catalog::PIPELINE_FASTQ_TO_BAM_DEFAULT => Ok(fastq_to_bam_default_profile()),
        _ => Err(anyhow!("unknown cross-domain profile: {id}")),
    }
}
