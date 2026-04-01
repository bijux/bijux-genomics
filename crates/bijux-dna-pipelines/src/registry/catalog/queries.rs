use crate::{Domain, PipelineProfile, StabilityTier};

use super::PipelineRegistry;

impl PipelineRegistry {
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
