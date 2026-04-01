//! Pipeline registry lookups by domain and id.

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::id_catalog;

use crate::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use crate::vcf::{vcf_minimal_profile, vcf_reference_basic_profile};
use crate::{Domain, PipelineProfile};

/// # Errors
/// Returns an error if the requested profile id is unknown for the domain.
pub fn profile_by_id(domain: Domain, id: &str) -> Result<PipelineProfile> {
    match domain {
        Domain::Fastq => crate::fastq::fastq_profiles_by_id(id),
        Domain::Bam => crate::bam::bam_profiles_by_id(id),
        Domain::Cross => match id {
            id_catalog::PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN => {
                Ok(fastq_to_bam_adna_shotgun_profile())
            }
            id_catalog::PIPELINE_FASTQ_TO_BAM_DEFAULT => Ok(fastq_to_bam_default_profile()),
            _ => Err(anyhow!("unknown cross-domain profile: {id}")),
        },
        Domain::Vcf => match id {
            id_catalog::PIPELINE_VCF_MINIMAL => Ok(vcf_minimal_profile()),
            id_catalog::PIPELINE_VCF_REFERENCE_BASIC => Ok(vcf_reference_basic_profile()),
            _ => Err(anyhow!("unknown VCF profile: {id}")),
        },
    }
}
