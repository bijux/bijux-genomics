//! Admit governed QC cohort metrics.
use crate::qc::model::LocalVcfQcSummary;
use anyhow::bail;
use anyhow::Result;

// These are exact expected values for the governed finite fixture, not approximate acceptance tolerances.
#[allow(clippy::float_cmp)]
pub(super) fn validate(summary: &LocalVcfQcSummary) -> Result<()> {
    if summary.sample_missingness_exclusion_threshold != 0.5 {
        bail!(
            "governed VCF QC smoke expected sample exclusion threshold 0.5, found {}",
            summary.sample_missingness_exclusion_threshold
        );
    }
    if summary.variant_missingness_exclusion_threshold != 0.5 {
        bail!(
            "governed VCF QC smoke expected variant exclusion threshold 0.5, found {}",
            summary.variant_missingness_exclusion_threshold
        );
    }
    if summary.maf_summary.observed_variant_count != 4 {
        bail!(
            "governed VCF QC smoke expected 4 observed variants, found {}",
            summary.maf_summary.observed_variant_count
        );
    }
    if summary.heterozygosity.heterozygous_call_count != 4 {
        bail!(
            "governed VCF QC smoke expected 4 heterozygous calls, found {}",
            summary.heterozygosity.heterozygous_call_count
        );
    }
    if summary.heterozygosity.homozygous_alt_call_count != 2 {
        bail!(
            "governed VCF QC smoke expected 2 homozygous-alt calls, found {}",
            summary.heterozygosity.homozygous_alt_call_count
        );
    }
    if summary.heterozygosity.het_hom_ratio != Some(2.0) {
        bail!(
            "governed VCF QC smoke expected het/hom ratio 2.0, found {:?}",
            summary.heterozygosity.het_hom_ratio
        );
    }
    if summary.hwe_summary.tested_variant_count != 3 {
        bail!(
            "governed VCF QC smoke expected 3 HWE-tested variants, found {}",
            summary.hwe_summary.tested_variant_count
        );
    }
    if summary.hwe_summary.pvalue_mean != Some(0.825_656) {
        bail!(
            "governed VCF QC smoke expected HWE p-value mean 0.825656, found {:?}",
            summary.hwe_summary.pvalue_mean
        );
    }
    if summary.hwe_summary.status != "computed_modern" {
        bail!(
            "governed VCF QC smoke expected HWE status `computed_modern`, found `{}`",
            summary.hwe_summary.status
        );
    }

    Ok(())
}
