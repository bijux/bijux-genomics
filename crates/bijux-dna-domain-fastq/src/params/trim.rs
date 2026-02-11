use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DamageMode, PairedMode};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub min_len: u32,
    #[serde(default)]
    pub q_cutoff: Option<u32>,
    pub adapter_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_mode: Option<DamageMode>,
    #[serde(default)]
    pub polyx_policy: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
    #[serde(default)]
    pub contaminant_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrimAdapterMode {
    Auto,
    Explicit,
    Linked,
    Anchored,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrimQualityMode {
    None,
    ThreePrime,
    FivePrime,
    BothEnds,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlapCollapseMode {
    Disabled,
    MergeOnly,
    CollapseConsensus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadHandlingMode {
    SingleEnd,
    PairedEnd,
    AutoDetect,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SkewerTrimParamsV1 {
    pub adapter_mode: TrimAdapterMode,
    pub min_length_bp: u32,
    pub quality_mode: TrimQualityMode,
    pub quality_cutoff_phred: u8,
    pub overlap_collapse: OverlapCollapseMode,
    pub read_handling: ReadHandlingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LeeHomTrimParamsV1 {
    pub adapter_mode: TrimAdapterMode,
    pub min_length_bp: u32,
    pub quality_mode: TrimQualityMode,
    pub quality_cutoff_phred: u8,
    pub overlap_collapse: OverlapCollapseMode,
    pub read_handling: ReadHandlingMode,
    pub min_overlap_bp: u32,
    pub allow_reverse_complement_overlap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AlienTrimmerParamsV1 {
    pub adapter_mode: TrimAdapterMode,
    pub min_length_bp: u32,
    pub quality_mode: TrimQualityMode,
    pub quality_cutoff_phred: u8,
    pub overlap_collapse: OverlapCollapseMode,
    pub read_handling: ReadHandlingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FastxClipperParamsV1 {
    pub adapter_mode: TrimAdapterMode,
    pub min_length_bp: u32,
    pub quality_mode: TrimQualityMode,
    pub quality_cutoff_phred: u8,
    pub overlap_collapse: OverlapCollapseMode,
    pub read_handling: ReadHandlingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum TrimToolParamsV1 {
    Skewer(SkewerTrimParamsV1),
    LeeHom(LeeHomTrimParamsV1),
    AlienTrimmer(AlienTrimmerParamsV1),
    FastxClipper(FastxClipperParamsV1),
}

impl TrimEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.adapter_policy.trim().is_empty() {
            missing.push("adapter_policy");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "min_len": self.min_len,
            "q": self.q_cutoff,
            "adapter_policy": self.adapter_policy,
            "damage_mode": self.damage_mode,
            "polyx_policy": self.polyx_policy,
            "n_policy": self.n_policy,
            "contaminant_policy": self.contaminant_policy,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
