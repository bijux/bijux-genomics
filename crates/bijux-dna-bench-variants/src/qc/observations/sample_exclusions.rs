//! Admit governed QC sample exclusions.
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::EXPECTED_EXCLUDED_SAMPLE_ID;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

// These are exact expected values for the governed finite fixture, not approximate acceptance tolerances.
#[allow(clippy::float_cmp)]
pub(super) fn validate(summary: &LocalVcfQcSummary) -> Result<()> {
    let qc_sparse = summary
        .sample_missingness
        .iter()
        .find(|row| row.sample_id == EXPECTED_EXCLUDED_SAMPLE_ID)
        .ok_or_else(|| {
            anyhow!("governed VCF QC smoke is missing sample `{EXPECTED_EXCLUDED_SAMPLE_ID}`")
        })?;
    if qc_sparse.missingness != 0.75 {
        bail!(
            "governed VCF QC smoke expected `{EXPECTED_EXCLUDED_SAMPLE_ID}` missingness 0.75, found {}",
            qc_sparse.missingness
        );
    }
    if summary.excluded_samples.len() != 1
        || summary.excluded_samples[0].sample_id != EXPECTED_EXCLUDED_SAMPLE_ID
    {
        bail!(
            "governed VCF QC smoke expected exactly one excluded sample `{EXPECTED_EXCLUDED_SAMPLE_ID}`, found {:?}",
            summary
                .excluded_samples
                .iter()
                .map(|row| row.sample_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    Ok(())
}
