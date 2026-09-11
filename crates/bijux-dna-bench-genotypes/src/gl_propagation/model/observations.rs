//! Private scientific observations consumed by metrics.
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LikelihoodFieldSummary {
    pub(crate) fields: BTreeSet<String>,
    pub(crate) site_count: u64,
}
