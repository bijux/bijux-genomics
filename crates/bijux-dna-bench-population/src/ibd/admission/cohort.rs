//! Owner: bijux-dna-bench-population
//! Ibd admission cohort.
use anyhow::bail;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeSet;

pub(crate) fn validate_expected_cohort_samples(
    sample_metadata: &[SampleMetadataRow],
    expected_samples: &[String],
) -> Result<()> {
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::<String>::new();
    for row in sample_metadata {
        if !expected_sample_set.contains(&row.sample_id) {
            continue;
        }
        if row.role != "cohort" {
            bail!(
                "governed VCF IBD smoke expected cohort role for `{}`, found `{}`",
                row.sample_id,
                row.role
            );
        }
        if !seen.insert(row.sample_id.clone()) {
            bail!("governed VCF IBD smoke metadata duplicated sample `{}`", row.sample_id);
        }
    }
    for sample_id in expected_samples {
        if !seen.contains(sample_id) {
            bail!("governed VCF IBD smoke metadata is missing expected sample `{sample_id}`");
        }
    }
    Ok(())
}
