//! Pipeline registry for FASTQ, BAM, and cross-domain profiles.

use anyhow::{anyhow, Result};

use crate::bam::{bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile};
use crate::cross::fastq_to_bam_adna_profile;
use crate::fastq::{fastq_default_profile, fastq_minimal_profile, DefaultPipelineOptions};
use crate::{Domain, PipelineProfile};

#[must_use]
pub fn fastq_profiles() -> Vec<PipelineProfile> {
    vec![
        fastq_default_profile(DefaultPipelineOptions::default()),
        fastq_minimal_profile(),
    ]
}

#[must_use]
pub fn bam_profiles() -> Vec<PipelineProfile> {
    vec![
        bam_default_profile(),
        bam_adna_shotgun_profile(),
        bam_adna_capture_profile(),
    ]
}

#[must_use]
pub fn cross_profiles() -> Vec<PipelineProfile> {
    vec![fastq_to_bam_adna_profile()]
}

/// # Errors
/// Returns an error if the requested profile id is unknown for the domain.
pub fn profile_by_id(domain: Domain, id: &str) -> Result<PipelineProfile> {
    match domain {
        Domain::Fastq => super::fastq::fastq_profiles_by_id(id),
        Domain::Bam => super::bam::bam_profiles_by_id(id),
        Domain::Cross => match id {
            "fastq-to-bam-adna" => Ok(fastq_to_bam_adna_profile()),
            _ => Err(anyhow!("unknown cross-domain profile: {id}")),
        },
        Domain::Vcf => Err(anyhow!("VCF pipelines not yet defined")),
    }
}
