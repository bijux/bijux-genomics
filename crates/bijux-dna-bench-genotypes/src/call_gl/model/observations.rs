//! Private scientific observations consumed by metrics.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LikelihoodSummary {
    pub(crate) likelihood_field: String,
    pub(crate) sites_with_likelihoods: u64,
    pub(crate) samples_with_likelihoods: u64,
    pub(crate) missing_likelihoods: u64,
}
