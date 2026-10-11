//! Owner: bijux-dna-bench-population
//! Pca observations evidence.
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn validate_pca_manifest_samples(
    manifest: &serde_json::Value,
    expected_samples: &[String],
) -> Result<()> {
    let manifest_sample_count = manifest
        .get("sample_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("PCA manifest missing `sample_count`"))?;
    let expected_sample_count =
        u64::try_from(expected_samples.len()).map_err(|_| anyhow!("sample count overflow"))?;
    if manifest_sample_count != expected_sample_count {
        bail!(
            "PCA manifest sample_count drifted: expected {expected_sample_count}, found {manifest_sample_count}"
        );
    }
    let manifest_sample_ids = manifest
        .get("sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("PCA manifest missing `sample_ids`"))?
        .iter()
        .map(|row| {
            row.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("PCA manifest sample_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    if manifest_sample_ids != expected_samples {
        bail!(
            "PCA manifest sample_ids drifted: expected {expected_samples:?}, found {manifest_sample_ids:?}"
        );
    }
    let manifest_sample_labels = manifest
        .get("sample_population_labels")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("PCA manifest missing `sample_population_labels`"))?;
    if manifest_sample_labels.len() != expected_samples.len() {
        bail!(
            "PCA manifest sample_population_labels length drifted: expected {}, found {}",
            expected_samples.len(),
            manifest_sample_labels.len()
        );
    }
    Ok(())
}
