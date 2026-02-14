use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PairedMode;
use crate::pipeline_contract::FastqPipelineMode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LibraryDamageTreatment {
    Udg,
    PartialUdg,
    NoUdg,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreprocessEffectiveParams {
    #[serde(default = "default_pipeline_mode")]
    pub pipeline_mode: FastqPipelineMode,
    pub paired_mode: PairedMode,
    pub library_declared_paired: bool,
    pub library_damage_treatment: LibraryDamageTreatment,
    pub threads: u32,
    pub stages: Vec<String>,
    #[serde(default)]
    pub enable_contaminant_removal: bool,
}

fn default_pipeline_mode() -> FastqPipelineMode {
    FastqPipelineMode::Shotgun
}

impl PreprocessEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.library_damage_treatment == LibraryDamageTreatment::Unknown {
            missing.push("library_damage_treatment");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.stages.is_empty() {
            missing.push("stages");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "pipeline_mode": self.pipeline_mode,
            "paired_mode": self.paired_mode,
            "library_declared_paired": self.library_declared_paired,
            "library_damage_treatment": self.library_damage_treatment,
            "threads": self.threads,
            "stages": self.stages,
            "enable_contaminant_removal": self.enable_contaminant_removal,
        })
    }
}
