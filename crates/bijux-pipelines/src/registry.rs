//! Pipeline registry for FASTQ, BAM, and cross-domain profiles.

use anyhow::{anyhow, Result};

use crate::bam::{bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile};
use crate::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use crate::fastq::{fastq_default_profile, fastq_minimal_profile, DefaultPipelineOptions};
use crate::{Domain, PipelineProfile, StabilityTier};

#[derive(Debug, Clone)]
pub struct PipelineRegistry {
    profiles: Vec<PipelineProfile>,
}

impl PipelineRegistry {
    #[must_use]
    pub fn v1() -> Self {
        let mut profiles = Vec::new();
        profiles.extend(fastq_profiles());
        profiles.extend(bam_profiles());
        profiles.extend(cross_profiles());
        Self { profiles }
    }

    #[must_use]
    pub fn list(&self, include_experimental: bool) -> Vec<&PipelineProfile> {
        self.profiles
            .iter()
            .filter(|profile| include_experimental || profile.stability == StabilityTier::Stable)
            .collect()
    }

    #[must_use]
    pub fn list_for_domain(
        &self,
        domain: Domain,
        include_experimental: bool,
    ) -> Vec<&PipelineProfile> {
        self.list(include_experimental)
            .into_iter()
            .filter(|profile| profile.domains.contains(&domain))
            .collect()
    }
}

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
    vec![
        fastq_to_bam_adna_shotgun_profile(),
        fastq_to_bam_default_profile(),
    ]
}

/// # Errors
/// Returns an error if the requested profile id is unknown for the domain.
pub fn profile_by_id(domain: Domain, id: &str) -> Result<PipelineProfile> {
    match domain {
        Domain::Fastq => super::fastq::fastq_profiles_by_id(id),
        Domain::Bam => super::bam::bam_profiles_by_id(id),
        Domain::Cross => match id {
            "fastq-to-bam__adna_shotgun__v1" => Ok(fastq_to_bam_adna_shotgun_profile()),
            "fastq-to-bam__default__v1" => Ok(fastq_to_bam_default_profile()),
            _ => Err(anyhow!("unknown cross-domain profile: {id}")),
        },
        Domain::Vcf => Err(anyhow!("VCF pipelines not yet defined")),
    }
}
