//! Scientific observation records consumed by metric and report projections.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilteredOutputSummary {
    pub(crate) pass_variants: u64,
    pub(crate) failed_variants: u64,
    pub(crate) filter_ids: Vec<String>,
}
