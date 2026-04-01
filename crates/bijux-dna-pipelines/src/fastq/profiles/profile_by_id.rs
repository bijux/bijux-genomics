use anyhow::anyhow;
use bijux_dna_core::prelude::id_catalog;

use super::{
    fastq_adna_profile, fastq_default_profile, fastq_minimal_profile, fastq_reference_adna_profile,
};
use crate::PipelineProfile;

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn fastq_profiles_by_id(id: &str) -> anyhow::Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_FASTQ_DEFAULT => Ok(fastq_default_profile()),
        id_catalog::PIPELINE_FASTQ_MINIMAL => Ok(fastq_minimal_profile()),
        id_catalog::PIPELINE_FASTQ_ADNA => Ok(fastq_adna_profile()),
        id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA => Ok(fastq_reference_adna_profile()),
        _ => Err(anyhow!("unknown FASTQ profile: {id}")),
    }
}
