use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::contract::ArtifactSpec;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ArtifactWriter;

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
        let checksums =
            bijux_dna_runtime::recording::write_artifact_checksums_json(stage_root, &output_spec)?;
        Ok(serde_json::to_value(checksums)?)
    }

    /// # Errors
    /// Returns an error if the stage manifest cannot be written atomically.
    pub fn write_stage_manifest(path: &Path, payload: &serde_json::Value) -> Result<PathBuf> {
        bijux_dna_infra::atomic_write_json(path, payload)?;
        Ok(path.to_path_buf())
    }

    /// # Errors
    /// Returns an error if checksums cannot be generated or stage manifest cannot be written.
    pub fn write_stage_outputs_and_manifest(
        stage_root: &Path,
        outputs: &[ArtifactSpec],
        stage_manifest_path: &Path,
        mut stage_manifest: serde_json::Value,
    ) -> Result<(serde_json::Value, PathBuf)> {
        let checksums = Self::write_output_checksums(stage_root, outputs)?;
        if let Some(obj) = stage_manifest.as_object_mut() {
            obj.insert("output_checksums".to_string(), checksums.clone());
        } else {
            return Err(anyhow::anyhow!(
                "stage manifest payload must be a JSON object"
            ));
        }
        let manifest_path = Self::write_stage_manifest(stage_manifest_path, &stage_manifest)?;
        Ok((checksums, manifest_path))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MetricsWriter;

impl MetricsWriter {
    #[must_use]
    pub fn required_keys_for_stage(stage_id: &str) -> Vec<&'static str> {
        let mut keys = vec![
            "schema_version",
            "stage_id",
            "tool_id",
            "runtime_s",
            "wall_time_ms",
            "exit_code",
        ];
        if stage_id.starts_with("bam.") {
            keys.push("memory_mb");
        } else if stage_id.starts_with("vcf.") {
            keys.push("records_in");
            keys.push("records_out");
        } else if stage_id.starts_with("fastq.") {
            keys.push("reads_in");
            keys.push("reads_out");
        }
        keys
    }

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

    /// # Errors
    /// Returns an error if stage-backed key validation fails or writing is unsuccessful.
    pub fn write_stage_metrics(
        path: &Path,
        stage_id: &str,
        metrics: &serde_json::Value,
    ) -> Result<PathBuf> {
        let keys = Self::required_keys_for_stage(stage_id);
        Self::write_metrics(path, metrics, &keys)
    }
}
