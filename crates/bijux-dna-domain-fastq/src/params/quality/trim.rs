use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DamageMode, PairedMode};

pub const TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION: &str = "bijux.fastq.params.trim_terminal_damage.v1";
pub const TRIM_POLYG_TAILS_SCHEMA_VERSION: &str = "bijux.fastq.params.trim_polyg_tails.v1";
pub const DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES: u32 = 2;
pub const DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES: u32 = 2;

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimTerminalDamageParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub damage_mode: DamageMode,
    #[serde(default = "default_terminal_damage_execution_policy")]
    pub execution_policy: TerminalDamageExecutionPolicy,
    pub trim_5p_bases: u32,
    pub trim_3p_bases: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_trim_5p_bases: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_trim_3p_bases: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalDamageExecutionPolicy {
    ExplicitTerminalTrim,
    PreserveUdgTrimmedEnds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTerminalDamagePolicy {
    pub execution_policy: TerminalDamageExecutionPolicy,
    pub effective_trim_5p_bases: u32,
    pub effective_trim_3p_bases: u32,
    pub requested_trim_5p_bases: u32,
    pub requested_trim_3p_bases: u32,
}

#[must_use]
pub fn default_terminal_damage_execution_policy() -> TerminalDamageExecutionPolicy {
    TerminalDamageExecutionPolicy::ExplicitTerminalTrim
}

#[must_use]
pub fn resolve_terminal_damage_policy(
    damage_mode: DamageMode,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
) -> ResolvedTerminalDamagePolicy {
    let preserves_udg_trimmed_ends = damage_mode == DamageMode::UdgTrimmed
        && trim_5p_bases == DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES
        && trim_3p_bases == DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES;
    if preserves_udg_trimmed_ends {
        return ResolvedTerminalDamagePolicy {
            execution_policy: TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds,
            effective_trim_5p_bases: 0,
            effective_trim_3p_bases: 0,
            requested_trim_5p_bases: trim_5p_bases,
            requested_trim_3p_bases: trim_3p_bases,
        };
    }
    ResolvedTerminalDamagePolicy {
        execution_policy: TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
        effective_trim_5p_bases: trim_5p_bases,
        effective_trim_3p_bases: trim_3p_bases,
        requested_trim_5p_bases: trim_5p_bases,
        requested_trim_3p_bases: trim_3p_bases,
    }
}

impl TrimTerminalDamageParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "damage_mode": self.damage_mode,
            "execution_policy": self.execution_policy,
            "trim_5p_bases": self.trim_5p_bases,
            "trim_3p_bases": self.trim_3p_bases,
            "requested_trim_5p_bases": self.requested_trim_5p_bases,
            "requested_trim_3p_bases": self.requested_trim_3p_bases,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimPolygTailsParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub trim_polyg: bool,
    pub min_polyg_run: u32,
}

impl TrimPolygTailsParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.min_polyg_run == 0 {
            missing.push("min_polyg_run");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "trim_polyg": self.trim_polyg,
            "min_polyg_run": self.min_polyg_run,
        })
    }
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
