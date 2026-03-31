use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::prelude::input_assessment::FastqLayout;
use bijux_dna_core::prelude::{CacheKey, Result as CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEnvironment {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub runner: String,
    pub platform: String,
    pub tool_images: Vec<ToolImageDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolImageDigest {
    pub tool: String,
    pub image: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStageEntry {
    pub stage_id: String,
    pub tool_id: String,
    pub execution_metrics_path: PathBuf,
    pub domain_metrics_path: PathBuf,
    pub logs_dir: PathBuf,
    pub outputs_dir: PathBuf,
    pub tool_invocation_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: String,
    pub contract_version: ContractVersion,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub pipeline: String,
    pub graph_hash: String,
    #[serde(default)]
    pub cache_key: Option<CacheKey>,
    pub layout: FastqLayout,
    pub stages: Vec<RunStageEntry>,
    #[serde(default)]
    pub tool_invocations: Vec<bijux_dna_core::metrics::ToolInvocationV1>,
    #[serde(default)]
    pub artifacts: Vec<RunArtifactEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifactEntry {
    pub name: String,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexEntry {
    pub run_id: String,
    pub domain: String,
    pub pipeline: String,
    pub stages: Vec<String>,
    pub layout: FastqLayout,
    pub tools: Vec<String>,
    pub objective: Option<String>,
    pub platform: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexLine {
    pub schema_version: u32,
    pub run: RunIndexEntry,
}

#[derive(Debug, Clone)]
pub struct RunLayout {
    pub run_dir: PathBuf,
    pub stages_dir: PathBuf,
    pub summary_dir: PathBuf,
    pub assessment_path: PathBuf,
    pub manifest_path: PathBuf,
    pub environment_path: PathBuf,
    pub metadata_path: PathBuf,
    pub events_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunLayoutV1 {
    pub schema_version: String,
    pub run_dir: String,
    pub stages_dir: String,
    pub summary_dir: String,
    pub assessment_path: String,
    pub manifest_path: String,
    pub environment_path: String,
    pub metadata_path: String,
    pub events_path: String,
}

impl RunLayout {
    #[must_use]
    pub fn contract(&self) -> RunLayoutV1 {
        RunLayoutV1 {
            schema_version: "bijux.run_layout.v1".to_string(),
            run_dir: self.run_dir.display().to_string(),
            stages_dir: self.stages_dir.display().to_string(),
            summary_dir: self.summary_dir.display().to_string(),
            assessment_path: self.assessment_path.display().to_string(),
            manifest_path: self.manifest_path.display().to_string(),
            environment_path: self.environment_path.display().to_string(),
            metadata_path: self.metadata_path.display().to_string(),
            events_path: self.events_path.display().to_string(),
        }
    }
}

impl RunManifest {
    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate(&self) -> CoreResult<()> {
        if self.graph_hash.trim().is_empty() {
            return Err(bijux_dna_core::prelude::BijuxError::validation(
                "run manifest graph_hash is empty",
            ));
        }
        if self.artifacts.is_empty() {
            return Err(bijux_dna_core::prelude::BijuxError::validation(
                "run manifest artifacts list is empty",
            ));
        }
        for artifact in &self.artifacts {
            if artifact.name.trim().is_empty() {
                return Err(bijux_dna_core::prelude::BijuxError::validation(
                    "run manifest artifact name is empty",
                ));
            }
            if artifact.sha256.trim().is_empty() {
                return Err(bijux_dna_core::prelude::BijuxError::validation(
                    "run manifest artifact hash is empty",
                ));
            }
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if canonical serialization fails.
    pub fn hash(&self) -> CoreResult<String> {
        let bytes = to_canonical_json_bytes(self)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}
