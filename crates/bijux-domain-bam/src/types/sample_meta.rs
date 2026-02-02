//! Sample metadata for BAM processing.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LibraryType {
    NonUdg,
    HalfUdg,
    Udg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strandedness {
    Unstranded,
    Forward,
    Reverse,
    Dual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaptureType {
    Shotgun,
    Capture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedSex {
    XX,
    XY,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Illumina,
    Bgi,
    IonTorrent,
    Nanopore,
    Pacbio,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReadGroupPolicy {
    Preserve,
    Merge,
    Regenerate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SampleMeta {
    pub library_type: LibraryType,
    pub strandedness: Strandedness,
    pub capture_type: CaptureType,
    #[serde(default)]
    pub expected_sex: Option<ExpectedSex>,
    pub platform: Platform,
    pub read_group_policy: ReadGroupPolicy,
}
