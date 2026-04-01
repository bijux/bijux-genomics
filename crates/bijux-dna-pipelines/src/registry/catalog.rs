//! Pipeline registry catalog assembly.

use super::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles};
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
        profiles.extend(vcf_profiles());
        profiles.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
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
            .filter(|profile| {
                profile.input_domains.contains(&domain) || profile.output_domains.contains(&domain)
            })
            .collect()
    }
}
