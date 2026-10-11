//! Owner: bijux-dna-bench-population
//! Admixture observations evidence.
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn validate_admixture_manifest(
    manifest: &serde_json::Value,
    expected_samples: &[String],
    selected_k: usize,
    cluster_headers: &[String],
) -> Result<()> {
    let manifest_sample_ids = manifest
        .get("sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("admixture manifest missing `sample_ids`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("admixture manifest sample_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    if manifest_sample_ids != expected_samples {
        bail!(
            "admixture manifest sample_ids drifted: expected {expected_samples:?}, found {manifest_sample_ids:?}"
        );
    }
    if cluster_headers.len() != selected_k {
        bail!(
            "admixture manifest cluster_headers length drifted: expected {}, found {}",
            selected_k,
            cluster_headers.len()
        );
    }
    let manifest_sample_labels = manifest
        .get("sample_population_labels")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("admixture manifest missing `sample_population_labels`"))?;
    if manifest_sample_labels.len() != expected_samples.len() {
        bail!(
            "admixture manifest sample_population_labels length drifted: expected {}, found {}",
            expected_samples.len(),
            manifest_sample_labels.len()
        );
    }
    Ok(())
}
