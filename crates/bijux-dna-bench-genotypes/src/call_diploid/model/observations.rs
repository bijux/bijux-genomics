//! Private scientific observations consumed by metrics.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiploidGenotypeSummary {
    pub(crate) called_genotypes: u64,
    pub(crate) heterozygous_count: u64,
    pub(crate) homozygous_ref_count: u64,
    pub(crate) homozygous_alt_count: u64,
    pub(crate) missing_count: u64,
}
