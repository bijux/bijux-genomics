//! Scientific observation records consumed by metric and report projections.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PostprocessOutputSummary {
    pub(crate) record_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) mq_removed: bool,
    pub(crate) multiallelic_records_remaining: u64,
}
