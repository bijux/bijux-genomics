/// Compare called genotypes when phase is outside the truth claim.
///
/// Exact strings remain equal. Otherwise both calls must contain at least two
/// non-missing alleles; allele order and `/` versus `|` encoding are ignored.
#[must_use]
pub fn genotypes_equivalent_unphased(expected: &str, observed: &str) -> bool {
    if expected == observed {
        return true;
    }
    let Some(expected_alleles) = normalized_unphased_alleles(expected) else {
        return false;
    };
    let Some(observed_alleles) = normalized_unphased_alleles(observed) else {
        return false;
    };
    expected_alleles == observed_alleles
}

fn normalized_unphased_alleles(genotype: &str) -> Option<Vec<&str>> {
    let mut alleles = genotype.split(['/', '|']).collect::<Vec<_>>();
    if alleles.len() < 2 || alleles.iter().any(|allele| allele.is_empty() || *allele == ".") {
        return None;
    }
    alleles.sort_unstable();
    Some(alleles)
}

#[cfg(test)]
mod tests {
    use super::genotypes_equivalent_unphased;

    #[test]
    fn unphased_truth_accepts_equivalent_phased_calls() {
        assert!(genotypes_equivalent_unphased("0/1", "0|1"));
        assert!(genotypes_equivalent_unphased("0/1", "1|0"));
        assert!(genotypes_equivalent_unphased("1/1", "1|1"));
    }

    #[test]
    fn unphased_truth_rejects_discordant_or_missing_calls() {
        assert!(!genotypes_equivalent_unphased("0/1", "0|0"));
        assert!(!genotypes_equivalent_unphased("0/1", ".|."));
        assert!(!genotypes_equivalent_unphased("0/1", "0|."));
        assert!(!genotypes_equivalent_unphased("0/1", "0"));
    }
}
