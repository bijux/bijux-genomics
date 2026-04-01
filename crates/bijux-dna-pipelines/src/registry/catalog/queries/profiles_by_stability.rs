use crate::registry::PipelineRegistry;
use crate::{PipelineProfile, StabilityTier};

impl PipelineRegistry {
    #[must_use]
    pub fn list(&self, include_experimental: bool) -> Vec<&PipelineProfile> {
        self.profiles
            .iter()
            .filter(|profile| include_experimental || profile.stability == StabilityTier::Stable)
            .collect()
    }
}
