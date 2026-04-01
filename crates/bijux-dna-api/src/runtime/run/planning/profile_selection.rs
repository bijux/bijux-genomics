use super::{Domain, PipelineProfile, PipelineRegistry, Result};

/// # Errors
/// Returns an error if the profile id is unknown.
pub fn select_pipeline(domain: Domain, profile_id: &str) -> Result<PipelineProfile> {
    bijux_dna_pipelines::registry::profile_by_id(domain, profile_id)
}

#[must_use]
pub fn select_pipelines(
    domain: Option<Domain>,
    include_experimental: bool,
) -> Vec<PipelineProfile> {
    let registry = PipelineRegistry::v1();
    if let Some(domain) = domain {
        registry
            .list_for_domain(domain, include_experimental)
            .into_iter()
            .cloned()
            .collect()
    } else {
        registry
            .list(include_experimental)
            .into_iter()
            .cloned()
            .collect()
    }
}
