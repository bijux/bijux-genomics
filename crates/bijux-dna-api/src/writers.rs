use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::contract::ArtifactSpec;

#[derive(Debug, Default, Clone, Copy)]
pub struct ArtifactWriter;

impl ArtifactWriter {
    /// # Errors
    /// Returns an error if checksums cannot be generated or persisted.
    pub fn write_output_checksums(
        stage_root: &Path,
        outputs: &[ArtifactSpec],
    ) -> Result<serde_json::Value> {
        let output_spec = outputs
            .iter()
            .map(|artifact| (artifact.name.to_string(), artifact.path.clone()))
            .collect::<Vec<_>>();
        let checksums = bijux_dna_runtime::recording::write_artifact_checksums_json(stage_root, &output_spec)?;
        Ok(serde_json::to_value(checksums)?)
    }

    /// # Errors
    /// Returns an error if the stage manifest cannot be written atomically.
    pub fn write_stage_manifest(path: &Path, payload: &serde_json::Value) -> Result<PathBuf> {
        bijux_dna_infra::atomic_write_json(path, payload)?;
        Ok(path.to_path_buf())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MetricsWriter;

impl MetricsWriter {
    /// # Errors
    /// Returns an error if required keys are missing.
    pub fn validate_required_keys(
        metrics: &serde_json::Value,
        required_keys: &[&str],
    ) -> Result<()> {
        let obj = metrics
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("metrics payload must be a JSON object"))?;
        let missing = required_keys
            .iter()
            .copied()
            .filter(|key| !obj.contains_key(*key))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(anyhow::anyhow!(
                "metrics schema violation: missing required keys {}",
                missing.join(", ")
            ));
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if validation fails or writing is unsuccessful.
    pub fn write_metrics(
        path: &Path,
        metrics: &serde_json::Value,
        required_keys: &[&str],
    ) -> Result<PathBuf> {
        Self::validate_required_keys(metrics, required_keys)?;
        bijux_dna_infra::atomic_write_json(path, metrics)?;
        Ok(path.to_path_buf())
    }
}
