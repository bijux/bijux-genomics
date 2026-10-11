//! Private scientific observations consumed by metrics.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PseudohaploidSiteSummary {
    pub(crate) target: u64,
    pub(crate) covered: u64,
    pub(crate) called: u64,
    pub(crate) missing: u64,
}
