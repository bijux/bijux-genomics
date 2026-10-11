//! Admit governed QC variant exclusions.
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::EXPECTED_EXCLUDED_VARIANT_ID;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

// These are exact expected values for the governed finite fixture, not approximate acceptance tolerances.
#[allow(clippy::float_cmp)]
pub(super) fn validate(summary: &LocalVcfQcSummary) -> Result<()> {
    let excluded_variant = summary
        .variant_missingness
        .iter()
        .find(|row| row.variant_id == EXPECTED_EXCLUDED_VARIANT_ID)
        .ok_or_else(|| {
            anyhow!("governed VCF QC smoke is missing variant `{EXPECTED_EXCLUDED_VARIANT_ID}`")
        })?;
    if excluded_variant.missingness != (2.0 / 3.0) {
        bail!(
            "governed VCF QC smoke expected `{EXPECTED_EXCLUDED_VARIANT_ID}` missingness {}, found {}",
            2.0 / 3.0,
            excluded_variant.missingness
        );
    }
    if summary.excluded_variants.len() != 1
        || summary.excluded_variants[0].variant_id != EXPECTED_EXCLUDED_VARIANT_ID
    {
        bail!(
            "governed VCF QC smoke expected exactly one excluded variant `{EXPECTED_EXCLUDED_VARIANT_ID}`, found {:?}",
            summary
                .excluded_variants
                .iter()
                .map(|row| row.variant_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    Ok(())
}
