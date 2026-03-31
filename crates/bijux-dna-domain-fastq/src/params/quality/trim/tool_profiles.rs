use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
