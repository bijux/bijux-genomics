use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageResultStatus {
    Ok,
    Refused,
    Failed,
    SkippedCached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifactEnvelopeV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub status: StageResultStatus,
    pub reason_code: String,
    pub manifest_json: PathBuf,
    pub metrics_json: PathBuf,
    pub checksums: PathBuf,
    pub provenance: PathBuf,
    pub logs: PathBuf,
}

/// # Errors
/// Returns an error if envelope cannot be serialized.
pub fn write_run_artifact_envelope(
    stage_root: &Path,
    stage_id: &str,
    status: StageResultStatus,
    reason_code: &str,
) -> Result<PathBuf> {
    let envelope = RunArtifactEnvelopeV1 {
        schema_version: "bijux.run_artifact_envelope.v1".to_string(),
        stage_id: stage_id.to_string(),
        status,
        reason_code: reason_code.to_string(),
        manifest_json: stage_root.join("stage_manifest.json"),
        metrics_json: stage_root.join("metrics.json"),
        checksums: stage_root.join("checksums.sha256"),
        provenance: stage_root.join("runtime_provenance.json"),
        logs: stage_root.join("logs"),
    };
    let path = stage_root.join("run_artifact_envelope.json");
    super::io::write_canonical_json(&path, &envelope)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}
