//! Observe VCF sample-column cardinality without conflating counting and validation.
use anyhow::{anyhow, Result};

/// Count sample columns in the first VCF column header.
///
/// This observation preserves empty or repeated sample columns. Callers requiring
/// valid sample identities must separately apply semantic VCF validation.
///
/// # Errors
/// Returns an error if the column header is absent or its count cannot fit in u64.
pub fn sample_count_from_vcf_text(raw: &str) -> Result<u64> {
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("VCF output is missing the #CHROM header"))?;
    let sample_count = sample_header.split('\t').skip(9).count();
    u64::try_from(sample_count).map_err(|_| anyhow!("VCF output sample count overflowed u64"))
}
